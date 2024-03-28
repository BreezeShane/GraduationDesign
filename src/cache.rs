use axum::{extract::{Multipart, Path as axum_Path}, http::StatusCode};
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use std::path::{Path, PathBuf};
use std::fs::create_dir;

const USER_PIC_PATH: &str = "./data_src/";

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "RequestUploadPic")]
pub struct RequestUploadPic {
    user_id: String   
}

fn obtain_user_directory(user_email: &String) -> Option<PathBuf> {
    let user_dir_name = BASE64_STANDARD.encode(user_email);

    let path = Path::new(USER_PIC_PATH);
    let user_dir_path = path.join(user_dir_name);
    if !user_dir_path.exists() {
        match create_dir(&user_dir_path) {
            Ok(_) => {
                println!("Directory initialized.");
            },
            Err(e) => {
                println!("Error creating directory: {}", e);
                return None;
            }
        }
    }
    return Some(user_dir_path.to_path_buf());
}

pub async fn handler_recieve_uploaded_pic(
    axum_Path(user): axum_Path<RequestUploadPic>, mut multipart: Multipart
)-> Result<(StatusCode, String), (StatusCode, String)> {
    if let Some(file) = multipart.next_field().await.unwrap() {
        let filename = file.file_name().unwrap().to_string();
        let data = file.bytes().await.unwrap();

        let file_path = obtain_user_directory(&user.user_id).unwrap();
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