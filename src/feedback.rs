use std::path::PathBuf;

use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use axum::{extract::State, http::StatusCode, Form};
use deadpool_postgres::Pool;
use postgres::types::ToSql;
use serde::{Deserialize, Serialize};
use tokio_postgres::row::Row;

use crate::authenticator::{check_permission, Permission};
use crate::doc_database::{TrainingTask, QueueTrait};
use crate::io_cache::obtain_dir;
use crate::config::FEEDBACK_EXPIRATION;
use crate::MultiState;


#[derive(Serialize, Deserialize)]
pub struct  RequestFeedback {
    useremail: String,
    pic_name: String,
    real_label: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct AccRejFeedback {
    useremail: String,
    pic_path: String,
    real_label: String,
    accept: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Feedback {
    timestamp: i64,
    from_user_email: String,
    deadline: Option<i64>,
    pic_path: String,
    real_label: Option<String>,
    acceptable: bool
}

impl Feedback {
    fn from_row_ref(row: &Row, trainable: bool) -> Self {
        let mut fb = Feedback {
            timestamp: row.get("time_stamp"),
            from_user_email: row.get("from_user_email"),
            deadline: None,
            pic_path: row.get("pic_path"),
            real_label: None,
            acceptable: false
        };
        if trainable {
            fb.deadline = row.get("time_out");
            fb.real_label = row.get("real_label");
        }
        fb
    }
}

pub async fn handler_subm_fb(
    State(multi_state): State<MultiState>,
    Form(user_feedback): Form<RequestFeedback>
) -> Result<(StatusCode, String), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_feedback.useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let (feedback, insert_statement) = (|| -> (Feedback, &str) {
        let feedback;
        let insert_statement;
        let pic_path = 
            PathBuf::from(
                obtain_dir(&user_feedback.useremail)
                .unwrap()
            )
                .join(user_feedback.pic_name).
                into_os_string().into_string().unwrap();

        match user_feedback.real_label {
            None => {
                feedback = Feedback {
                    timestamp: Utc::now().timestamp(),
                    from_user_email: user_feedback.useremail.clone(),
                    pic_path,
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
                    from_user_email: user_feedback.useremail.clone(),
                    deadline: Some(Utc::now().timestamp() + FEEDBACK_EXPIRATION),
                    pic_path,
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
    //                     from_user_email: user_feedback.from_user_email.clone(),
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

    let client = multi_state.db_pool.get().await.unwrap();
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

pub async fn handler_fetch_all_fb(
    State(multi_state): State<MultiState>,
    Query(user_id): Query<String>
) -> Result<Response, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_id, Permission::MngFeedBack).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let vec_tfbs = _fetch_fb(&multi_state.db_pool, true).await;
    let vec_ufbs =  _fetch_fb(&multi_state.db_pool, false).await;

    return Ok(
        Json((vec_tfbs, vec_ufbs)).into_response()
    );
}

async fn _fetch_fb(pool: &Pool, trainable: bool) -> Vec<Feedback> {
    let client = pool.get().await.unwrap();
    let query_str = match trainable {
        true => "
            SELECT time_stamp,from_user_email,time_out,pic_link,real_label,acceptable FROM TFeedback;
        ",
        false => "
            SELECT time_stamp,from_user_email,pic_link,acceptable FROM UFeedback;
        "
    };

    let query_tfb_statement = client
        .prepare(query_str)
        .await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap();

    let vec_fb: Vec<Feedback> = client
        .query(&query_tfb_statement, &[])
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap() 
        .iter()
        .map(|row| Feedback::from_row_ref(row, trainable))
        .collect::<Vec<Feedback>>();

    vec_fb
}

pub async fn handler_acc_rej_fb(
    State(multi_state): State<MultiState>,
    Form(acc_rej_fb): Form<AccRejFeedback>
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &acc_rej_fb.useremail, Permission::MngFeedBack).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let client = multi_state.db_pool.get().await.unwrap();
    let del_statement = client
    .prepare("
        DELETE FROM TFeedback WHERE pic_link='$1';
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let pic_path = acc_rej_fb.pic_path;
    let label = acc_rej_fb.real_label;
    
    if acc_rej_fb.accept {
        let task = TrainingTask {
            pic_path: pic_path.clone(),
            label: label.clone()
        };
        let mut queue = multi_state.train_queue.lock().unwrap();
        let _ = queue.append_task(task);
    }
    
    let row = client
        .execute(&del_statement, &[&pic_path])
        .await
        .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;

    if row < 1 {
        return Err((StatusCode::NOT_MODIFIED, "Remove trainable data row failed".to_string()));
    }

    Ok(())
}


pub async fn handler_fetch_ufb(
    State(multi_state): State<MultiState>,
    Query(user_id): Query<String>
) -> Result<Response, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &user_id, Permission::MngFeedBack).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let vec_ufbs =  _fetch_fb(&multi_state.db_pool, false).await;

    return Ok(
        Json(vec_ufbs).into_response()
    );
}

pub async fn handler_label_pic(
    State(multi_state): State<MultiState>,
    Form(request_feedback): Form<RequestFeedback>
) -> Result<(), (StatusCode, String)> {
    if let None = request_feedback.real_label {
        return Err((StatusCode::CONFLICT, "Not set the label!".to_string()));
    }

    let fb = Feedback {
        timestamp: Utc::now().timestamp(),
        from_user_email: request_feedback.useremail,
        deadline: Some(Utc::now().timestamp() + FEEDBACK_EXPIRATION),
        pic_path: request_feedback.pic_name,
        real_label: request_feedback.real_label,
        acceptable: false
    };

    let client = multi_state.db_pool.get().await.unwrap();
    let del_statement = client
    .prepare("
        DELETE FROM UFeedback WHERE pic_link='$1';
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let insert_statement = client
    .prepare("
        INSERT INTO TFeedback (time_stamp, from_user_email, time_out, pic_link, real_label, acceptable)
        VALUES
        ($1, $2, $3, $4, $5, $6)
    ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let insert_row = client
            .execute(&insert_statement, 
                &[
                    &fb.timestamp, &fb.from_user_email, &fb.deadline, 
                    &fb.pic_path, &fb.real_label, &fb.acceptable
                ])
            .await
            .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()));

    if let Ok(ins_count) = insert_row {
        if ins_count < 1 {
            return Err((StatusCode::NOT_MODIFIED, "Failed to insert new trainable feedback!".to_string()));
        }
        let delet_row = client
        .execute(&del_statement, 
            &[
                &fb.pic_path
            ])
        .await
        .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
        if delet_row < 1 {
            return Err((StatusCode::NOT_MODIFIED, "Failed to delete original untrainable feedback!".to_string()));
        }
    }
    Ok(())
}