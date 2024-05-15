use std::path::PathBuf;

use axum::{extract::{Path, State}, http::StatusCode, Form};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use crate::{authenticator::{check_permission, Permission}, config::USER_PIC_PATH, custom_hash_map::SPECIES_HASHMAP, io_agent::_obtain_dir, MultiState};
use crate::config::DL_SVC_HOST;

#[derive(Serialize, Deserialize)]
pub struct RequestInfer {
    useremail: String,
    file_list: String // JSON Serialized Vec<String>
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper (table = "Wiki")]
pub struct ResponseInferResultUnit {
    file_name: String,
    specie_name: String,
    content: String
}

type ResponseInferResult = Vec<ResponseInferResultUnit>;

pub async fn handler_infer(
    State(multi_state): State<MultiState>,
    Form(user_inference): Form<RequestInfer>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_inference.useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let files_vec: Vec<String> = serde_json::from_str(&user_inference.file_list).unwrap();
    tracing::warn!("files_vec: {files_vec:#?}");

    let mut result_res: ResponseInferResult = Vec::new();

    let infer_path_root = PathBuf::from(USER_PIC_PATH);
    let infer_path = infer_path_root.join(_obtain_dir(&user_inference.useremail).unwrap());
    for file_name in files_vec {
        let image_path = infer_path.join(file_name);
        let image_result = py_inference(image_path);
        result_res.push(image_result);
    }

    let json_string = serde_json::to_string(&result_res).unwrap();
    return Ok(json_string);
    // Example Response
    // let response : ResponseInferResult = vec![
    //     ResponseInferResultUnit { file_name: "./39181.jpg".to_string(), specie_name: "odontothrips loti".to_string(), content: "whatever".to_string() },
    //     ResponseInferResultUnit { file_name: "./58237.jpg".to_string(), specie_name: "Erythroneura apicalis".to_string(), content: "whatever you say".to_string() },
    //     ResponseInferResultUnit { file_name: "./66871.jpg".to_string(), specie_name: "Dasineura sp".to_string(), content: "whatever I shout".to_string() },
    // ];
}

fn py_inference(image_path: PathBuf) -> ResponseInferResultUnit {
    // TODO: Call Python function and get the inference result response.
    let (spec, text) = SPECIES_HASHMAP.get("rice leaf roller").unwrap();
    ResponseInferResultUnit {
        file_name: image_path.into_os_string().into_string().unwrap(),
        specie_name: spec.to_owned().to_string(),
        content: text.to_owned().to_string()
    }
}

pub async fn handler_authenticate_ssh(
    State(multi_state): State<MultiState>,
    Path(useremail): Path<String>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &useremail, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let ssh_addr = String::from(DL_SVC_HOST);
    return Ok(ssh_addr);
}