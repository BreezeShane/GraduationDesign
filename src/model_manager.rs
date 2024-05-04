use std::{env, fs};

use axum::{extract::State, http::StatusCode, Form, Json};
use chrono::{DateTime, Local};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

use crate::{authenticator::{check_permission, Permission}, io_agent::{backup_models, _path_is_valid, remove_models}, MultiState};

#[derive(Deserialize, Serialize)]
pub struct RequestFetchModels {
    useremail: String,
    request_dir: String,
}

#[derive(Deserialize, Serialize)]
pub struct FileMetadata {
    file_name: String,
    file_type: String,
    file_size: u64,
    last_access_time: String,
    last_modified_time: String,
    creation_date: String,
}

#[derive(Deserialize, Serialize)]
pub struct RequestFileOperation {
    useremail: String,
    operation_type: String,
    files2operate: String // Json String
}

// #[debug_handler]
pub async fn handler_fetch_all_models(
    State(multi_state): State<MultiState>,
    Form(request): Form<RequestFetchModels>
) -> Result<Json<Vec<FileMetadata>>, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request.useremail, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    if !_path_is_valid(&request.request_dir) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }
    let mut file_list: Vec<FileMetadata> = Vec::new();
    let current_dir = env::current_dir().unwrap().join(&request.request_dir);
    for entry in fs::read_dir(current_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let curr_file_metadata = fs::metadata(&path).unwrap();
        let access_time: DateTime<Local> = curr_file_metadata.accessed().unwrap().into();
        let modified_time: DateTime<Local> = curr_file_metadata.modified().unwrap().into();
        let creation_date: DateTime<Local> = curr_file_metadata.created().unwrap().into();
        let file_metadata = FileMetadata {
            file_name: entry.file_name().into_string().unwrap(),
            file_type: path.extension().unwrap().to_owned().into_string().unwrap(),
            file_size: curr_file_metadata.len(),
            last_access_time: access_time.to_string(),
            last_modified_time: modified_time.to_string(),
            creation_date: creation_date.to_string(),
        };
        file_list.push(file_metadata);
    }
    Ok(Json(file_list))
}

pub async fn handler_file_operation(
    State(multi_state): State<MultiState>,
    Form(file_operation_request): Form<RequestFileOperation>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &file_operation_request.useremail, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let operation_type = file_operation_request.operation_type.as_str();
    let files2operate: Vec<String> = serde_json::from_str(file_operation_request.files2operate.as_str()).unwrap();
    match operation_type {
        "backup" => {
            let write_bytes = backup_models(files2operate.iter()).map_err(|err| (StatusCode::CONFLICT, format!("Failed to backup files! Get Error: {err}").to_owned())).await?;
            return Ok(format!("File operations finished! Written {write_bytes} bytes!"));
        },
        "remove" => {
            let count_of_files = remove_models(files2operate.iter()).map_err(|err| (StatusCode::CONFLICT, format!("Failed to backup files! Get Error: {err}").to_owned())).await?;
            return Ok(format!("File operations finished! Removed {count_of_files} files!"));
        },
        _ => {
            return Err((StatusCode::FORBIDDEN, "Operation was not permitted or implemented! Only support backup and remove files.".to_string()));
        }
    }
}

// pub async fn handler_rm_dset(
//     State(multi_state): State<MultiState>,
//     Path((user_id, dataset_name)): Path<(String, String)>
// ) -> Result<(), (StatusCode, String)> {
//     if !check_permission(&multi_state.db_pool, &user_id, Permission::MngModel).await.unwrap() {
//         return Err(
//             (StatusCode::FORBIDDEN, "Not permitted!".to_string())
//         );
//     }

//     let mut dataset_manager = multi_state.dset_db.lock().unwrap();
//     dataset_manager
//         .rm_dset(&dataset_name)
//         .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))
// }