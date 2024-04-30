use axum::{extract::{Path, State}, http::StatusCode, Form, Json};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::{authenticator::{check_permission, role_to_string, Permission, ProofAccount}, MultiState};

#[derive(Serialize, Deserialize)]
enum Action {
    BanAccount,
    UnbanAccount,
    AddUser
}

#[derive(Serialize, Deserialize)]
pub struct RequestUserManagement {
    action: Action,
    admin_name: String,
    admin_email: String,
    permissions: i8,
    expiration_of_banning: Option<i64>,
    operated_user_email: Option<String>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "Account")]
pub struct UserInfo {
    nick_name: String,
    email: String,
    contribution: i16,
    permissions: i16
}

#[derive(Serialize, Deserialize)]
pub struct ResponseUserInfo {
    nick_name: String,
    email: String,
    contribution: String,
    role: String,
}

pub async fn handler_ban_or_unban_user(
    State(multi_state): State<MultiState>,
    Form(action_request): Form<RequestUserManagement>
) -> Result<axum::Json<String>, (StatusCode, String)> {
    if let Action::BanAccount | Action::UnbanAccount = action_request.action {
        if !check_permission(&multi_state.db_pool, &action_request.admin_email, Permission::MngUsr).await.unwrap() {
            return  Err(
                (StatusCode::FORBIDDEN, "Not permitted!".to_string())
            );
        }

        let client = multi_state.db_pool.get().await.unwrap();
        let query_statement = client
        .prepare("
            SELECT email, permissions, available FROM account WHERE email=$1;
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, format!("Bad query! {}", err)))?;

        let user_to_operate = client
            .query(&query_statement, &[&action_request.operated_user_email])
            .await
            .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
            .iter()
            .map(|row| ProofAccount::from_row_ref(row).unwrap())
            .collect::<Vec<ProofAccount>>()
            .pop()
            .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", action_request.operated_user_email)));

        if let Ok(user) = user_to_operate {
            let user_status = user.available;
            let update_statement = client
                .prepare("
                    UPDATE account SET available=$1 WHERE email=$2
                ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

            let modified_count = client
                .execute(&update_statement, &[&(!user_status), &user.email])
                .await
                .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;

            return Ok(Json(modified_count.to_string()));
        }
    }
    return Err((StatusCode::BAD_REQUEST, "The api could not process the request!".to_string()));
}

pub async fn handler_user_info(
    State(multi_state): State<MultiState>,
    Path(useremail): Path<String>,
) -> Result<axum::Json<ResponseUserInfo>, (StatusCode, String)> {
    let client = multi_state.db_pool.get().await.unwrap();
    let query_statement = client
    .prepare("
        SELECT nick_name, email, contribution, permissions FROM account WHERE email=$1;
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, format!("Bad query! {}", err)))?;

    let user = client
        .query(&query_statement, &[&useremail])
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
        .iter()
        .map(|row| UserInfo::from_row_ref(row).unwrap())
        .collect::<Vec<UserInfo>>()
        .pop()
        .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", useremail)))?;

    let response = ResponseUserInfo {
        nick_name: user.nick_name,
        email: user.email,
        contribution: user.contribution.to_string(),
        role: role_to_string(user.permissions)
    };

    Ok(Json(response))
}