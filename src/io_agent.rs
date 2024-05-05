use axum::{extract::{Query, State}, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}};
use std::fs::create_dir;
use axum::{
    extract::{Multipart, Path as RoutePath},
    http::StatusCode,
};
use std::{io::{Error, ErrorKind}, path::{Path, PathBuf}, slice::Iter};

use crate::config::{MODEL_BACKUP_STORED_PATH, MODEL_STORED_PATH, UFEEDBACK_STORED_DIRECTORY, USER_PIC_PATH};
use crate::authenticator::{check_permission, Permission};
use crate::MultiState;

#[derive(Serialize, Deserialize)]
pub struct RequestImageFetch {
    useremail: String,
    image_name: String
}

pub async fn handler_fetch_image(
    State(multi_state): State<MultiState>,
    Query(request_image_fetch): Query<RequestImageFetch>
) -> Result<Response, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request_image_fetch.useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let image_pathbuf = PathBuf::from(UFEEDBACK_STORED_DIRECTORY).join(request_image_fetch.image_name);
    let mut image_file_handle = tokio::fs::File::open(&image_pathbuf).await.unwrap();
    let mut buffer = vec![];
    image_file_handle.read_to_end(&mut buffer).await.unwrap();

    Ok(buffer.into_response())
}

pub fn _path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

pub fn _obtain_dir(user_email: &str) -> Result<String, std::ffi::OsString>{
    let path_buffer = __obtain_dir(user_email).unwrap();
    path_buffer.into_os_string().into_string()
}

fn __obtain_dir(user_email: &str) -> Result<PathBuf, String> {
    let user_dir_name = _generate_user_folder_name(user_email);

    let path = Path::new(USER_PIC_PATH);
    let user_dir_path = path.join(user_dir_name);
    if !user_dir_path.exists() {
        match create_dir(&user_dir_path) {
            Ok(_) => {
                tracing::info!("{:#?} Directory initialized.", user_dir_path);
            },
            Err(e) => {
                tracing::error!("Error creating directory: {}", e);
                return Err(format!("Error creating directory: {}", e));
            }
        }
    }
    return Ok(user_dir_path);
}

pub fn _generate_user_folder_name(user_email: &str) -> String {
    hex::encode(user_email).to_owned()
}

pub fn _generate_new_file_name(user_email: &str, file_name: &str) -> String {
    let mut new_file_name = _generate_user_folder_name(user_email);
    new_file_name.push_str("_");
    new_file_name.push_str(file_name);
    return new_file_name;
}

pub async fn handler_upload_pic(
    State(multi_state): State<MultiState>,
    RoutePath(useremail): RoutePath<String>,
    mut multipart: Multipart
)-> Result<(StatusCode, String), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    if let Some(file) = multipart.next_field().await.unwrap() {
        let filename = file.file_name().unwrap().to_string();
        let data = file.bytes().await.unwrap();

        let file_path = __obtain_dir(&useremail).unwrap();
        let upload_path = file_path.join(&filename);
        //std::fs::write(&filename, &data).map_err(|err| err.to_string())?;
        tokio::fs::write(&upload_path, &data)
            .await
            .map_err(|err| (StatusCode::NOT_ACCEPTABLE, err.to_string()))?;

        return Ok((StatusCode::OK, format!(
            "【上传的文件】文件名：{:?}, 文件大小：{}",
            filename,
            data.len()
        )))
    }
    Err((StatusCode::NOT_ACCEPTABLE, "没有上传文件或上传文件不合法".to_string()))
}

pub async fn backup_models(files: Iter<'_, String>) -> tokio::io::Result<u64> {
    let src_dir_path = PathBuf::from(MODEL_STORED_PATH);
    let dest_dir_path = PathBuf::from(MODEL_BACKUP_STORED_PATH);
    let total_write = __copy_files(files, &src_dir_path, &dest_dir_path).await;
    return total_write;
}

pub async fn remove_models(files: Iter<'_, String>) -> tokio::io::Result<u64> {
    let src_dir_path = PathBuf::from(MODEL_STORED_PATH);
    let count = __remove_files(files, &src_dir_path).await;
    count
}

pub async fn _move_image_in_fb(file_name: &str, src_dir_path: &PathBuf, dest_dir_path: &PathBuf) -> tokio::io::Result<u64> {
    let src_path = src_dir_path.join(file_name);
    let dest_path = dest_dir_path.join(file_name);
    tokio::fs::copy(&src_path, dest_path).await?;
    tokio::fs::remove_file(&src_path).await?;
    Ok(1)
}

pub async fn _rename_file(current_file_name: &str, new_file_name: &str, src_dir_path: &PathBuf) -> Result<(), Error> {
    let src_path = src_dir_path.join(current_file_name);
    let dest_path = src_dir_path.join(new_file_name);

    tokio::fs::rename(src_path, dest_path).await
}

pub fn __generate_pic_label_file(pic_location: &str) -> String {
    let mut label_file_pathbuf = PathBuf::from(pic_location);
    label_file_pathbuf.set_extension("txt");
    return label_file_pathbuf.to_str().unwrap().to_owned();
}

pub async fn create_and_write_label_file(file_name: &str, input_data: &[u8], dest_dir_path: &PathBuf)  -> tokio::io::Result<()> {
    let dest_path = dest_dir_path.join(file_name);
    let mut file = File::create(dest_path).await?;
    file.write_all(input_data).await?;
    Ok(())
}

async fn __copy_files(files: Iter<'_, String>, src_dir_path: &PathBuf, dest_dir_path: &PathBuf) -> tokio::io::Result<u64> {
    let mut total_write = 0;
    for file in files {
        let src_path = src_dir_path.join(file.as_str());
        let dest_path = dest_dir_path.join(file.as_str());
        let write_bytes = tokio::fs::copy(src_path, dest_path).await.unwrap();
        total_write += write_bytes;
    }
    Ok(total_write)
}

async fn __remove_files(files: Iter<'_, String>, src_dir_path: &PathBuf) -> tokio::io::Result<u64> {
    let mut count = 0;
    let expected_count = files.len() as u64;
    for file in files {
        let file_path = src_dir_path.join(file.as_str());

        tracing::warn!("Removing the file locate at {:#?}", file_path);

        tokio::fs::remove_file(file_path).await?;
        count += 1;
    }
    if count == expected_count {
        return Ok(count);
    } else {
        return Err(Error::new(
            ErrorKind::Interrupted,
            format!("Unable to remove files! Expected removed {} files, but removed {} files.",
            expected_count, count
        )));
    }
}

async fn __move_files(files: Iter<'_, String>, src_dir_path: &PathBuf, dest_dir_path: &PathBuf) -> tokio::io::Result<u64> {
    let files_to_remove = files.clone();
    match __copy_files(files, src_dir_path, dest_dir_path).await {
        Ok(_) => {
            __remove_files(files_to_remove, &src_dir_path).await
        },
        Err(e) => Err(e)
    }
}

// pub async fn handler_upload_dset(
//     State(multi_state): State<MultiState>,
//     RoutePath((user_id, file_name)): RoutePath<(String, String)>,
//     request: Request,
// ) -> Result<(), (StatusCode, String)> {
//     if !check_permission(&multi_state.db_pool, &user_id, Permission::MngModel).await.unwrap() {
//         return Err(
//             (StatusCode::FORBIDDEN, "Not permitted!".to_string())
//         );
//     }
//     let tracing_request = format!("The received request is: \n{request:#?}");
//     tracing::warn!(tracing_request);
//     let _ = stream_to_file(&file_name, request.into_body().into_data_stream()).await;

//     let mut dataset_writer = multi_state.dset_db.lock().unwrap();
//     let dataset = Dataset {
//         name: file_name,
//         timestamp: Local::now().timestamp(),
//         available: false
//     };
//     dataset_writer.append_dset(dataset).map_err(|_| (StatusCode::NOT_MODIFIED, "Failed to append dataset into Dataset Database!".to_string()))
// }

// async fn stream_to_file<S, E>(file_name: &str, stream: S) -> Result<(), (StatusCode, String)>
// where
//     S: Stream<Item = Result<Bytes, E>>,
//     E: Into<BoxError>,
// {
//     // To prevent directory traversal attack
//     if !path_is_valid(file_name) {
//         return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
//     }

//     async {
//         // Convert the stream into an `AsyncRead`.
//         let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
//         let body_reader = StreamReader::new(body_with_io_error);
//         futures::pin_mut!(body_reader);

//         // Create the file. `File` implements `AsyncWrite`.
//         let path = Path::new(DATASETS_DIRECTORY).join(file_name);
//         let mut file = BufWriter::new(File::create(path).await?);

//         // Copy the body into the file.
//         tokio::io::copy(&mut body_reader, &mut file).await?;

//         Ok::<_, io::Error>(())
//     }
//     .await
//     .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
// }