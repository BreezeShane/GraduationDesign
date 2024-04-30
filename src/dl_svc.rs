use axum::{extract::State, http::StatusCode, Form};
use serde::{Deserialize, Serialize};

use crate::{authenticator::{check_permission, Permission}, MultiState};

#[derive(Serialize, Deserialize)]
pub struct RequestInfer {
    useremail: String,
    file_list: String // JSON Serialized Vec<String>
}

type ResponseInferResult = Vec<(String, String)>;

pub async fn handler_infer(
    State(multi_state): State<MultiState>,
    Form(user_feedback): Form<RequestInfer>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_feedback.useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }


    // Example Response
    let response : ResponseInferResult = vec![
        ("./2024-03-31_18-46_1.png".to_string(), "flag".to_string()),
        ("./2024-03-31_18-46.png".to_string(), "wore".to_string()),
        ("./2024-03-31_18-47.png".to_string(), "mutate".to_string()),
    ];
    let json_string = serde_json::to_string(&response).unwrap();
    return Ok(json_string);
}