use axum::extract::State;
use base64::{prelude::BASE64_URL_SAFE, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use std::path::{Path, PathBuf};
use std::fs::create_dir;
use axum::{
    body::Bytes,
    extract::{Multipart, Path as RoutePath, Request},
    http::StatusCode,
    BoxError,
};
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use crate::authenticator::{check_permission, Permission};
use crate::config::{USER_PIC_PATH, DATASETS_DIRECTORY};
use crate::doc_database::{Dataset, DatasetTrait};
use crate::MultiState;

// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "RequestUploadPic")]
pub struct RequestUploadPic {
    user_id: String   
}

pub fn obtain_dir(user_email: &String) -> Result<String, std::ffi::OsString>{
    let path_buffer = __obtain_dir(&user_email).unwrap();
    path_buffer.into_os_string().into_string()
}

fn __obtain_dir(user_email: &String) -> Result<PathBuf, String> {
    let user_dir_name = BASE64_URL_SAFE.encode(user_email);

    let path = Path::new(USER_PIC_PATH);
    let user_dir_path = path.join(user_dir_name);
    if !user_dir_path.exists() {
        match create_dir(&user_dir_path) {
            Ok(_) => {
                println!("Directory initialized.");
            },
            Err(e) => {
                println!("Error creating directory: {}", e);
                return Err(format!("Error creating directory: {}", e));
            }
        }
    }
    return Ok(user_dir_path);
}

pub async fn handler_recieve_uploaded_pic(
    State(multi_state): State<MultiState>,
    RoutePath(user): RoutePath<RequestUploadPic>, 
    mut multipart: Multipart
)-> Result<(StatusCode, String), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user.user_id, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    if let Some(file) = multipart.next_field().await.unwrap() {
        let filename = file.file_name().unwrap().to_string();
        let data = file.bytes().await.unwrap();

        let file_path = __obtain_dir(&user.user_id).unwrap();
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

pub async fn handler_recieve_uploaded_dataset(
    State(multi_state): State<MultiState>,
    RoutePath((user_id, file_name)): RoutePath<(String, String)>,
    request: Request,
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_id, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let _ = stream_to_file(&file_name, request.into_body().into_data_stream()).await;

    let mut dataset_writer = multi_state.dset_db.lock().unwrap();
    let dataset = Dataset {
        name: file_name,
        timestamp: Utc::now().timestamp(),
        available: false
    };
    dataset_writer.append_dset(dataset).map_err(|_| (StatusCode::NOT_MODIFIED, "Failed to append dataset into Dataset Database!".to_string()))
}

async fn stream_to_file<S, E>(file_name: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // To prevent directory traversal attack
    if !path_is_valid(file_name) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = Path::new(DATASETS_DIRECTORY).join(file_name);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}