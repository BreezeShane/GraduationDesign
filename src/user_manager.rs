use axum::{extract::{Path, State}, http::StatusCode, Form, Json};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::{authenticator::{check_permission, encrypt_password, role_to_string, string_to_role, Permission, AccountUnit}, MultiState};

#[derive(Serialize, Deserialize)]
pub struct RequestUserManagement {
    admin_email: String,
    user_emails: String, // Json String
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

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestUserManageUnit {
    useremail: String
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "Account")]
pub struct UserManageUnit {
    nick_name: String,
    email: String,
    contribution: i16,
    permissions: i16,
    available: bool
}

#[derive(Serialize, Deserialize)]
pub struct ResponseUserManageUnit {
    username: String,
    useremail: String,
    user_contribution: i16,
    user_identity: String,
    available: bool
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper (table = "Account")]
struct User2Operate {
    email: String,
    available: bool
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper (table = "Account")]
struct Admin2Add {
    nick_name: String,
    password_salt: String,
    password_hash: String,
    email: String,
    permissions: i16
}

#[derive(Deserialize)]
pub struct RequestAdminAdd {
    admin_email: String,
    username: String,
    useremail: String,
    password: String,
    repassword: String,
    role: String
}

pub async fn handler_fetch_all_users(
    State(multi_state): State<MultiState>,
    Form(request): Form<RequestUserManageUnit>
) -> Result<Json<Vec<ResponseUserManageUnit>>, (StatusCode, String)> {
    let useremail = request.useremail;
    if !check_permission(&multi_state.db_pool, &useremail, Permission::MngUsr).await.unwrap() {
        return  Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let client = multi_state.db_pool.get().await.unwrap();
    let query_statement = client
        .prepare("
            SELECT nick_name, email, contribution, permissions, available FROM account;
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, format!("Bad query! {}", err)))?;

    let mut user_list: Vec<ResponseUserManageUnit> = Vec::new();
    let users = client
        .query(&query_statement, &[])
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
        .iter()
        .map(|row| UserManageUnit::from_row_ref(row).unwrap())
        .collect::<Vec<UserManageUnit>>();

    for user in users {
        if user.email == useremail {
            continue;
        }
        user_list.push(ResponseUserManageUnit{
            username: user.nick_name,
            useremail: user.email,
            user_contribution: user.contribution,
            user_identity: role_to_string(user.permissions),
            available: user.available
        })
    }
    return Ok(Json(user_list));
}

pub async fn handler_suspend_or_unsuspend_user(
    State(multi_state): State<MultiState>,
    Form(action_request): Form<RequestUserManagement>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &action_request.admin_email, Permission::MngUsr).await.unwrap() {
        return  Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let users_to_operate: Vec<String> = serde_json::from_str(action_request.user_emails.as_str()).unwrap();

    let tracing_string = format!("Gained deserialized obj is: {users_to_operate:#?}");
    tracing::warn!(tracing_string);

    let expected_total_count = users_to_operate.len() as u64;
    let client = multi_state.db_pool.get().await.unwrap();
    let query_statement = client
    .prepare("
        SELECT email, available FROM account WHERE email=$1;
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, format!("Bad query! {}", err)))?;

    let mut count_of_operation = 0;
    for useremail in users_to_operate {
        let user_to_operate = client
            .query(&query_statement, &[&useremail])
            .await
            .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
            .iter()
            .map(|row| User2Operate::from_row_ref(row).unwrap())
            .collect::<Vec<User2Operate>>()
            .pop()
            .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", useremail)));

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

            count_of_operation += modified_count;
        }
    }
    if count_of_operation == expected_total_count {
        return Ok("Operation finished!".to_string());
    } else {
        return Err((StatusCode::BAD_REQUEST,
            format!(
                "There is something worng! Need to process {expected_total_count} accounts, but only {count_of_operation} were done.",)));
    }
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

pub async fn handler_add_admin(
    State(multi_state): State<MultiState>,
    Form(request_add_admin): Form<RequestAdminAdd>
) -> Result<String, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request_add_admin.admin_email, Permission::MngUsr).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let client = multi_state.db_pool.get().await.unwrap();

    if request_add_admin.password != request_add_admin.repassword {
        return Err((StatusCode::NOT_ACCEPTABLE, "The passwords should be the same!".to_string()))
    }

    let query_statement = client
    .prepare("
        SELECT email FROM account WHERE email=$1;
    ")
    .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let account = client
    .query(&query_statement, &[&request_add_admin.useremail])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| AccountUnit::from_row_ref(row).unwrap())
    .collect::<Vec<AccountUnit>>()
    .pop();

    match account {
        None => {
            let contribute: i16 = 0;
            let available = true;
            let permissions = string_to_role(request_add_admin.role) as i16;
            let (passwd_salt, passwd_hash) =
                encrypt_password(request_add_admin.password);

            let insert_statement = client
            .prepare("
                INSERT INTO account (nick_name, password_salt, password_hash, email, contribution, available, permissions)
                VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

            // let rows = client
            // .execute(&insert_statement,
            //     &[ &user_request.username, &passwd_salt, &passwd_hash, &user_request.email,
            //         &contribute, &available, &permissions]
            // )
            // .await
            // .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
            let rows = client
            .execute(&insert_statement,
                &[ &request_add_admin.username, &passwd_salt, &passwd_hash, &request_add_admin.useremail,
                    &contribute, &available, &permissions]
            )
            .await
            .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;

            if rows < 1 {
                return Err((StatusCode::NOT_MODIFIED, "Register account failed".to_string()));
            }
            Ok("Succeeded to sign up an admin!".to_string())
        },
        Some(_) => {
            Err((StatusCode::CONFLICT, "The email has been used!".to_string()))
        }
    }
}