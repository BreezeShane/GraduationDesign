use chrono::Utc;
use axum::{extract::{Path, State}, http::StatusCode, Form};
use deadpool_postgres::Pool;
use postgres::types::ToSql;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::authenticator::{check_permission, Account, Permission};

const FEEDBACK_EXPIRATION: i64 = 86400;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "RequestFeedback")]
pub struct  RequestFeedback {
    user_id: String,
    pic_name: String,
    real_label: Option<String>
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "Feedback")]
pub struct Feedback {
    timestamp: i64,
    from_user_email: String,
    deadline: Option<i64>,
    pic_path: String,
    real_label: Option<String>,
    acceptable: bool
}

pub async fn handler_feedback(
    State(pool): State<Pool>,
    Path(request): Path<RequestFeedback>,
    Form(user_feedback): Form<Feedback>
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let client = pool.get().await.unwrap();

    let auth_statement = client
    .prepare("
        SELECT nick_name, email, permissions, available FROM account WHERE email=$1 ORDER BY id DESC LIMIT 1;
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let current_user = client
    .query(&auth_statement, &[&request.user_id])
    .await
    .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    .iter()
    .map(|row| Account::from_row_ref(row).unwrap())
    .collect::<Vec<Account>>()
    .pop()
    .ok_or((StatusCode::NOT_FOUND, format!("Couldn't find account: {:?}", request.user_id)))?;

    if !check_permission(&current_user, Permission::Common) {
        return  Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let (feedback, insert_statement) = (|| -> (Feedback, &str) {
        let feedback;
        let insert_statement;

        match user_feedback.real_label {
            None => {
                feedback = Feedback {
                    timestamp: Utc::now().timestamp(),
                    from_user_email: current_user.email.clone(),
                    pic_path: user_feedback.pic_path.clone(),
                    acceptable: false,
                    real_label: None,
                    deadline: None,
                };
                insert_statement = "
                    INSERT INTO UFeedback (time_stamp, from_user_id, pic_link, acceptable)
                    VALUES
                    ($1, $2, $3, $4)
                ";
            },
            Some(_) => {
                feedback = Feedback {
                    timestamp: Utc::now().timestamp(),
                    from_user_email: current_user.email.clone(),
                    deadline: Some(Utc::now().timestamp() + FEEDBACK_EXPIRATION),
                    pic_path: user_feedback.pic_path.clone(),
                    real_label: user_feedback.real_label,
                    acceptable: false
                };
                insert_statement = "
                    INSERT INTO TFeedback (time_stamp, from_user_id, time_out, pic_link, real_label, acceptable)
                    VALUES
                    ($1, $2, $3, $4, $5, $6)
                ";
            }
        }
        (feedback, insert_statement)
    })();

    let params: Vec<&(dyn ToSql + Sync)> = match feedback.real_label {
        None => vec![
            &feedback.timestamp,
            &feedback.from_user_email,
            &feedback.pic_path,
            &feedback.acceptable,
        ],
        Some(_) => vec![
            &feedback.timestamp,
            &feedback.from_user_email,
            &feedback.deadline,
            &feedback.pic_path,
            &feedback.real_label,
            &feedback.acceptable,
        ]
    };

    // let (insert_statement, params) = (
    //     || -> (&str, Vec<&(dyn ToSql + Sync)>) {
    //         let feedback;
    //         let insert_statement;
    //         let params: Vec<&(dyn ToSql + Sync)>;
    //         match user_feedback.real_label {
    //             None => {
    //                 feedback = Feedback {
    //                     timestamp: Utc::now().timestamp(),
    //                     from_user_email: current_user.email.clone(),
    //                     pic_path: user_feedback.pic_path.clone(),
    //                     acceptable: false,
    //                     real_label: None,
    //                     deadline: None,
    //                 };
    //                 insert_statement = "
    //                     INSERT INTO UFeedback (time_stamp, from_user_id, pic_link, acceptable)
    //                     VALUES
    //                     ($1, $2, $3, $4)
    //                 ";
    //                 params = vec![
    //                     &feedback.timestamp,
    //                     &feedback.from_user_email,
    //                     &feedback.pic_path,
    //                     &feedback.acceptable,
    //                 ];
    //             },
    //             Some(_) => {
    //                 feedback = Feedback {
    //                     timestamp: Utc::now().timestamp(),
    //                     from_user_email: current_user.email.clone(),
    //                     deadline: Some(Utc::now().timestamp() + FEEDBACK_EXPIRATION),
    //                     pic_path: user_feedback.pic_path.clone(),
    //                     real_label: user_feedback.real_label,
    //                     acceptable: false
    //                 };
    //                 insert_statement = "
    //                     INSERT INTO TFeedback (time_stamp, from_user_id, time_out, pic_link, real_label, acceptable)
    //                     VALUES
    //                     ($1, $2, $3, $4, $5, $6)
    //                 ";
    //                 params = vec![
    //                     &feedback.timestamp,
    //                     &feedback.from_user_email,
    //                     &feedback.deadline,
    //                     &feedback.pic_path,
    //                     &feedback.real_label,
    //                     &feedback.acceptable,
    //                 ]
    //             }
    //         }
    //         (insert_statement, params)
    // })();

    let feedback_statement = client
            .prepare(&insert_statement).await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?; 

    let rows = client
    .execute(&feedback_statement, &params)
    .await
    .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
    
    if rows < 1 {
        return Err((StatusCode::NOT_MODIFIED, "Insert feedback failed".to_string()));
    }
    
    Ok((StatusCode::OK, "Succeed to submit the feedback!".to_string()))
}