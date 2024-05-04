use std::path::PathBuf;

use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Local;
use axum::{extract::State, http::StatusCode, Form};
use deadpool_postgres::Pool;
use postgres::types::ToSql;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::row::Row;

use crate::authenticator::{check_permission, Permission};
use crate::io_agent::{create_and_write_label_file, _generate_new_file_name, _move_image_in_fb, _obtain_dir, _rename_file};
use crate::config::{DATA_TO_TRAIN_DIRECTORY, FEEDBACK_EXPIRATION, TFEEDBACK_STORED_DIRECTORY, UFEEDBACK_STORED_DIRECTORY};
use crate::MultiState;

#[derive(Serialize, Deserialize, Debug)]
pub struct  FeedbackFileUnit {
    filename: String,
    label: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct  RequestFeedback {
    useremail: String,
    file_with_label_list: String
}

#[derive(Serialize, Deserialize)]
pub struct  RequestLabelImage {
    useremail: String,
    image_name: String,
    image_label: String
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper (table = "UFeedback")]
pub struct LabelImageUnit {
    pic_link: String,
    real_label: String,
    submit_count: i64
}

#[derive(Serialize, Deserialize)]
pub struct AccRejFeedbackUnit {
    useremail: String,
    pic_path: String,
    real_label: String,
    accept: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AccRejFeedback {
    useremail: String,
    files_to_operate: Vec<AccRejFeedbackUnit>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Feedback {
    timestamp: i64,
    from_user_email: String,
    time_out: Option<i64>,
    pic_link: String,
    real_label: Option<String>,
    submit_count: i64,
    acceptable: bool
}

#[derive(Serialize, Deserialize, Debug, PostgresMapper)]
#[pg_mapper(table = "UFeedback")]
pub struct ResponseFeedbackUnit {
    pic_link: String,
}

#[derive(Deserialize)]
pub struct RequestEmail {
    email: String
}

impl Feedback {
    fn from_row_ref(row: &Row, trainable: bool) -> Self {
        let mut fb = Feedback {
            timestamp: row.get("time_stamp"),
            from_user_email: row.get("from_user_email"),
            time_out: None,
            pic_link: row.get("pic_link"),
            real_label: None,
            submit_count: row.get("submit_count"),
            acceptable: row.get("acceptable"),
        };
        if trainable {
            fb.time_out = row.get("time_out");
            fb.real_label = row.get("real_label");
        }
        fb
    }
}

pub async fn handler_subm_fb(
    State(multi_state): State<MultiState>,
    Form(user_feedback): Form<RequestFeedback>
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let useremail = user_feedback.useremail;
    if !check_permission(&multi_state.db_pool, &useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let files_with_label: Vec<FeedbackFileUnit> = serde_json::from_str(&user_feedback.file_with_label_list).unwrap();
    let client = multi_state.db_pool.get().await.unwrap();

    for item in files_with_label.iter() {
        let feedback_for_submission = __generate_feedback_and_move_file(item, useremail.as_str()).await.unwrap();
        let insert_statement = __generate_insert_statement(item);
        let params = __generate_params(&feedback_for_submission);

        let feedback_statement = client
            .prepare(insert_statement.as_str()).await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

        let rows = client
            .execute(&feedback_statement, &params)
            .await
            .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;

        if rows < 1 {
            return Err((StatusCode::NOT_MODIFIED, "Insert feedback failed".to_string()));
        }
    }
    Ok((StatusCode::OK, "Succeed to submit the feedback!".to_string()))
}

pub async fn handler_fetch_all_fb(
    State(multi_state): State<MultiState>,
    Query(get_request): Query<RequestEmail>
) -> Result<Response, (StatusCode, String)> {
    let useremail = get_request.email;
    if !check_permission(&multi_state.db_pool, &useremail, Permission::MngFeedBack).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let vec_tfbs = __fetch_fb(&multi_state.db_pool, true).await;
    // let vec_ufbs =  _fetch_fb(&multi_state.db_pool, false).await;

    return Ok(Json(vec_tfbs).into_response());
}

async fn __fetch_fb(pool: &Pool, trainable: bool) -> Vec<Feedback> {
    let client = pool.get().await.unwrap();
    let query_str = match trainable {
        true => "
            SELECT time_stamp, from_user_email, time_out, pic_link, real_label, acceptable FROM TFeedback;
        ",
        false => "
            SELECT time_stamp, from_user_email, pic_link, acceptable FROM UFeedback;
        "
    };

    let query_tfb_statement = client
        .prepare(query_str)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap();

    let vec_fb = client
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
    Form(request_fb): Form<AccRejFeedback>
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request_fb.useremail, Permission::MngFeedBack).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let files_with_label = request_fb.files_to_operate;
    let client = multi_state.db_pool.get().await.unwrap();
    let del_tfb_statement = client
        .prepare("
            DELETE FROM TFeedback WHERE pic_link='$1';
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let query_ufb_statement = client
        .prepare("
            SELECT pic_link FROM UFeedback WHERE pic_link='$1'
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let del_ufb_statement = client
        .prepare("
            DELETE FROM UFeedback WHERE pic_link='$1';
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let tfeedback_dir_path = PathBuf::from(TFEEDBACK_STORED_DIRECTORY);
    let data_to_train_dir_path = PathBuf::from(DATA_TO_TRAIN_DIRECTORY);


    for file in files_with_label.iter() {
        if file.accept {
            _move_image_in_fb(
                file.pic_path.as_str(),
                &tfeedback_dir_path,
                &data_to_train_dir_path
            ).await.unwrap();
            create_and_write_label_file(
                file.pic_path.as_str(),
                file.real_label.as_bytes(),
                &data_to_train_dir_path)
            .await.unwrap();

            let query_ufb_vec = client
                .query(&query_ufb_statement, &[&file.pic_path])
                .await
                .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap();
            if query_ufb_vec.len() > 0 {
                let del_ufb_row = client
                    .execute(&del_ufb_statement, &[&file.pic_path])
                    .await
                    .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
                if del_ufb_row < 1 {
                    return Err((StatusCode::NOT_MODIFIED, "Remove untrainable data row failed".to_string()));
                }
            }
        }
        let del_tfb_row = client
            .execute(&del_tfb_statement, &[&file.pic_path])
            .await
            .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
        if del_tfb_row < 1 {
            return Err((StatusCode::NOT_MODIFIED, "Remove trainable data row failed".to_string()));
        }
    }
    // if request_fb.accept {
    //     let task = TrainingTask {
    //         pic_path: pic_path.clone(),
    //         label: label.clone()
    //     };
    //     let mut queue = multi_state.train_queue.lock().unwrap();
    //     let _ = queue.append_task(task);
    // }
    Ok(())
}


pub async fn handler_fetch_ufb(
    State(multi_state): State<MultiState>,
    Query(request_fetch): Query<RequestEmail>
) -> Result<Response, (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request_fetch.email, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }

    let client = multi_state.db_pool.get().await.unwrap();
    let query_tfb_statement = client
        .prepare("
            SELECT pic_link FROM UFeedback;
        ")
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap();

    let vec_ufbs = client
        .query(&query_tfb_statement, &[])
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string())).unwrap()
        .iter()
        .map(|row| ResponseFeedbackUnit::from_row_ref(row).unwrap())
        .collect::<Vec<ResponseFeedbackUnit>>();

    return Ok(
        Json(vec_ufbs).into_response()
    );
}

pub async fn handler_label_pic(
    State(multi_state): State<MultiState>,
    Form(request_label_image): Form<RequestLabelImage>
) -> Result<(), (StatusCode, String)> {
    if !check_permission(&multi_state.db_pool, &request_label_image.useremail, Permission::Common).await.unwrap() {
        return Err(
            (StatusCode::FORBIDDEN, "Not permitted!".to_string())
        );
    }
    let image_pathbuf = PathBuf::from(UFEEDBACK_STORED_DIRECTORY).join(&request_label_image.image_name);
    let client = multi_state.db_pool.get().await.unwrap();

    let query_statement = client
        .prepare("
            SELECT pic_link, real_label, submit_count FROM TFeedback
            WHERE
            pic_link=$1 AND real_label=$2
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let update_statement = client
        .prepare("
            UPDATE TFeedback
            SET submit_count=$1
            WHERE pic_link=$2
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let insert_statement = client
        .prepare("
            INSERT INTO TFeedback (time_stamp, from_user_email, time_out, pic_link, real_label, submit_count, acceptable)
            VALUES
            ($1, $2, $3, $4, $5, $6, $7)
        ").await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let src_path = image_pathbuf.as_os_str().to_str().unwrap();

    let query_row = client
        .query(&query_statement, &[&src_path, &request_label_image.image_label])
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
        .iter()
        .map(|row| LabelImageUnit::from_row_ref(row).unwrap())
        .collect::<Vec<LabelImageUnit>>()
        .pop();

    match query_row {
        Some(label_image_unit) => {
            let new_count = label_image_unit.submit_count + 1;
            let update_row = client
                .execute(&update_statement, &[&new_count, &label_image_unit.pic_link])
                .await
                .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;
            if update_row > 0 {
                return Ok(());
            } else {
                return Err((StatusCode::NOT_MODIFIED, "Failed to update the record!".to_string()));
            }
        },
        None => {
            let feedback = Feedback {
                timestamp: Local::now().timestamp(),
                from_user_email: request_label_image.useremail.to_owned(),
                time_out: Some(Local::now().timestamp() + FEEDBACK_EXPIRATION),
                pic_link: src_path.to_owned(),
                real_label: Some(request_label_image.image_label.to_owned()),
                submit_count: 0,
                acceptable: false
            };

            let insert_row = client
                .execute(&insert_statement, &[
                    &feedback.timestamp, &feedback.from_user_email, &feedback.time_out,
                    &feedback.pic_link, &feedback.real_label, &feedback.submit_count, &feedback.acceptable
                ])
                .await
                .map_err(|err| (StatusCode::NOT_MODIFIED, err.to_string()))?;

            if insert_row > 0 {
                return Ok(());
            } else {
                return Err((StatusCode::NOT_MODIFIED, "Failed to update the record!".to_string()));
            }
        }
    }
}

async fn __generate_feedback_and_move_file(file_unit: &FeedbackFileUnit, useremail: &str) -> Result<Feedback, std::io::Error> {
    let ufeedback_dir_path = PathBuf::from(UFEEDBACK_STORED_DIRECTORY);
    let tfeedback_dir_path = PathBuf::from(TFEEDBACK_STORED_DIRECTORY);

    let src_dir_path = PathBuf::from(_obtain_dir(useremail).unwrap());

    if let Some(label) = file_unit.label.to_owned() {
        // TODO : Check if feedback uploaded exists
        _move_image_in_fb(file_unit.filename.as_str(), &src_dir_path, &tfeedback_dir_path).await?;
        let new_file_name = _generate_new_file_name(useremail, file_unit.filename.as_str());
        _rename_file(file_unit.filename.as_str(), new_file_name.as_str(), &tfeedback_dir_path).await?;

        return Ok(Feedback {
            timestamp: Local::now().timestamp(),
            from_user_email: useremail.to_string(),
            pic_link: new_file_name,
            time_out: Some(Local::now().timestamp() + FEEDBACK_EXPIRATION),
            real_label: Some(label),
            submit_count: 0,
            acceptable: false
        })
    } else {
        // TODO : Check if feedback uploaded exists
        _move_image_in_fb(file_unit.filename.as_str(), &src_dir_path, &ufeedback_dir_path).await?;
        let new_file_name = _generate_new_file_name(useremail, file_unit.filename.as_str());
        _rename_file(file_unit.filename.as_str(), new_file_name.as_str(), &ufeedback_dir_path).await?;

        return Ok(Feedback {
            timestamp: Local::now().timestamp(),
            from_user_email: useremail.to_string(),
            pic_link: new_file_name,
            time_out: None,
            real_label: None,
            submit_count: 0,
            acceptable: false
        });
    }
}

fn __generate_insert_statement(file_unit: &FeedbackFileUnit) -> String {
    let stmt = match file_unit.label {
        Some(_) => {
            "
                INSERT INTO TFeedback (time_stamp, from_user_email, time_out, pic_link, real_label, acceptable)
                VALUES
                ($1, $2, $3, $4, $5, $6)
            "
        },
        None => {
            "
                INSERT INTO UFeedback (time_stamp, from_user_email, pic_link, acceptable)
                VALUES
                ($1, $2, $3, $4)
            "
        }
    };
    return stmt.to_owned();
}

fn __generate_params<'a>(feedback: &'a Feedback) -> Vec<&'a (dyn ToSql + Sync)> {
    let params: Vec<&(dyn ToSql + Sync)> = match feedback.real_label {
        None => vec![
            &feedback.timestamp,
            &feedback.from_user_email,
            &feedback.pic_link,
            &feedback.acceptable,
        ],
        Some(_) => vec![
            &feedback.timestamp,
            &feedback.from_user_email,
            &feedback.time_out,
            &feedback.pic_link,
            &feedback.real_label,
            &feedback.submit_count,
            &feedback.acceptable,
        ]
    };
    return params;
}