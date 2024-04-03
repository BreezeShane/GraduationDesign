use axum::{extract::{Path, State}, http::StatusCode, Form};
use serde::{Deserialize, Serialize};

use crate::{authenticator::{check_permission, Permission}, doc_database::DatasetTrait, MultiState};

#[derive(Deserialize, Serialize)]
pub struct RequestXchDsetStat {
    user_id: String,
    dataset_name: String,
}

// #[debug_handler]
pub async fn handler_xch_dset_stat(
    State(multi_state): State<MultiState>,
    Form(request): Form<RequestXchDsetStat>
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request.user_id, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let mut dataset_manager = multi_state.dset_db.lock().unwrap();
    let result = dataset_manager.xch_stat(&request.dataset_name)
    .map_err(|err| (StatusCode::NOT_MODIFIED, err));
    
    match result {
        Ok(_) => return Ok(()),
        Err(err) => return Err(err)
    }
}

pub async fn handler_rm_dset(
    State(multi_state): State<MultiState>,
    Path((user_id, dataset_name)): Path<(String, String)>
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_id, Permission::MngModel).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let mut dataset_manager = multi_state.dset_db.lock().unwrap();
    dataset_manager
        .rm_dset(&dataset_name)
        .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))
}