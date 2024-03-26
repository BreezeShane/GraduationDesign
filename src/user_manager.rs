use axum::{extract::State, http::StatusCode, Form, Json};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::authenticator::{Account, Permissions};

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

pub async fn handler_ban_or_unban_user(
    State(pool): State<Pool>,
    Form(action_request): Form<RequestUserManagement>
) -> Result<axum::Json<String>, (StatusCode, String)> {
    if let Action::BanAccount | Action::UnbanAccount = action_request.action {
        let client = pool.get().await.unwrap();

        let query_statement = client
            .prepare("
                SELECT nick_name, email, permissions, available FROM account WHERE email=$1 ORDER BY id DESC LIMIT 1;
            ").await.map_err(|err| (StatusCode::BAD_REQUEST, format!("Bad query! {}", err)))?;

        let check_permissions = client
            .query(&query_statement, &[&action_request.admin_email])
            .await
            .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
            .iter()
            .map(|row| Account::from_row_ref(row).unwrap())
            .collect::<Vec<Account>>()
            .pop()
            .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", action_request.operated_user_email)))?;

        if check_permissions.permissions != Permissions::UserAdmin as i8 {
            return  Err(
                (StatusCode::FORBIDDEN, "Not permitted!".to_string())
            );
        }
            
        let user_to_operate = client
            .query(&query_statement, &[&action_request.operated_user_email])
            .await
            .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
            .iter()
            .map(|row| Account::from_row_ref(row).unwrap())
            .collect::<Vec<Account>>()
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