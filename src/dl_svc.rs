use std::path::PathBuf;

use axum::{extract::{Path, State}, http::StatusCode, Form};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use std::process::Command;

use crate::{
    authenticator::{check_permission, Permission},
    config::{COMPILED_MODEL_STORED_PATH, DL_SVC_HOST, USER_PIC_PATH},
    io_agent::_obtain_dir,
    species_vector::SPECIES_VECTOR,
    MultiState
};

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
        let image_path = infer_path.join(&file_name);
        let cmd_output = Command::new("python")
        .arg("../dl_svc/TransferProcedures/infer_by_tvm.py")
        .arg("optimized").arg("llvm")
        .arg(image_path.as_os_str().to_str().unwrap())
        .output().expect("failed to execute process");
        let label = String::from_utf8_lossy(&cmd_output.stdout).to_string().parse::<usize>().unwrap();
        let (_, (specie_name, content)) = SPECIES_VECTOR[label];
        result_res.push(ResponseInferResultUnit {
            file_name,
            specie_name: specie_name.to_string(),
            content: content.to_string()
        });
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