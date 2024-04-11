pub mod daemon;
pub mod config;
pub mod io_cache;
pub mod feedback;
pub mod user_manager;
pub mod doc_database;
pub mod model_manager;
pub mod authenticator;
pub mod dl_svc;

use std::{fs::copy, path::PathBuf, sync::{Arc, Mutex}};
//use tokio::sync::Mutex;
use authenticator::{handler_sign_in, handler_sign_out, handler_sign_up, middleware_authorize};
use chrono::Utc;
use daemon::{Cronie, Daemon};
use io_cache::{handler_upload_dset, handler_upload_pic};
use config::{DATASETS_STORED_PATH, QUEUE_STORED_PATH};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use feedback::{handler_acc_rej_fb, handler_fetch_all_fb, handler_fetch_ufb, handler_label_pic, handler_subm_fb};
use model_manager::handler_xch_dset_stat;
use postgres::Client;
use std::fs::read_dir;
use tokio_postgres::{Config, NoTls};
use axum::{
    extract::{DefaultBodyLimit, FromRef}, middleware, routing::{get, post}, Router
};
use user_manager::handler_ban_or_unban_user;
use doc_database::{
    DatasetVec, DatasetTrait,
    Queue, QueueTrait
};

use crate::config::{MODEL_BACKUP_STORED_PATH, MODEL_STORED_PATH};

// use axum_macros::debug_handler; // Important!

#[derive(Clone)]
pub struct MultiState {
    db_pool: Pool,
    dset_db: Arc<Mutex<DatasetVec>>,
    train_queue: Arc<Mutex<Queue>>
}
impl FromRef<MultiState> for Pool {
    fn from_ref(input: &MultiState) -> Self {
        input.db_pool.clone()
    }
}
impl FromRef<MultiState> for Arc<Mutex<DatasetVec>> {
    fn from_ref(input: &MultiState) -> Self {
        input.dset_db.clone()
    }
}
impl FromRef<MultiState> for Arc<Mutex<Queue>> {
    fn from_ref(input: &MultiState) -> Self {
        input.train_queue.clone()
    }
}


#[tokio::main]
async fn main() {
    let mut config = Config::new();
    config.host("localhost");
    config.user("postgres");
    config.password("postgres");
    config.dbname("InsectSys");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(config, NoTls, mgr_config);

    let multi_state = MultiState {
        db_pool: Pool::builder(mgr).max_size(16).build().unwrap(),
        dset_db: Arc::new(
            Mutex::new(
                DatasetVec::load(DATASETS_STORED_PATH).unwrap()
            )
        ),
        train_queue: Arc::new(
            Mutex::new(
                Queue::load(QUEUE_STORED_PATH).unwrap()
            )
        )
    };
    // build our application with a single route
    
    let app = Router::new() 
    // .route("/:user_id/result", get())
        .route("/:user_id/upload_pic", post(handler_upload_pic))
        .route("/user/label_pic", get(handler_fetch_ufb).post(handler_label_pic))
        .route("/user/subm_fb", post(handler_subm_fb))

    // .route("/admin/:user_id/", get())
        .route("/admin/feedback_manage", get(handler_fetch_all_fb).post(handler_acc_rej_fb))
        .route("/admin/user_manage", post(handler_ban_or_unban_user))
    
        .route("/admin/xch_dset_stat", post(handler_xch_dset_stat))
        .route("/admin/:user_id/dataset_manage/:file_name", post(handler_upload_dset))
        // .route("/admin/:user_id/training_panel", get())
        // .route("/admin/:user_id/model_backup", get())
        // .route("/admin/:user_id/training_effect", get())

        .route("/sign_out/:user_id", get(handler_sign_out))
        .route_layer(middleware::from_fn(middleware_authorize))
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sign_in", post(handler_sign_in))
        .route("/sign_up", post(handler_sign_up))
        .with_state(multi_state)
        .layer(DefaultBodyLimit::max(1024));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    let mut glob_daemon = Daemon::new();
    // let _ = glob_daemon.append_task("auto_rej_fd", Box::new(|pool| {
    //     Box::pin(auto_rej_fd(pool))
    // }));
    // let _ = glob_daemon.append_task("auto_bak_mod", Box::new(|_| {
    //     Box::pin(auto_bak_mod())
    // }));

    let _ = glob_daemon.append_task("auto_rej_fd", Box::new(|| -> Result<(), String> {
        let mut cli = Client::connect("postgresql://postgres:postgres@localhost/InsectSys", NoTls).unwrap();
        let query_result = cli.query("
            SELECT id, time_out FROM TFeedback;
        ", &[]).unwrap();
        let right_now = Utc::now().timestamp();
        for row in query_result {
            let row_id: i64 = row.get(0);
            let row_timeout: i64 = row.get(1);

            if row_timeout <= right_now {
                let _ = cli.execute("
                    DELETE FROM TFeedback WHERE id=$1;
                ", &[&row_id]);
            }
        }
        drop(cli);
        Ok(())
    }));
    let _ = glob_daemon.append_task("auto_bak_mod", Box::new(|| -> Result<(), String> {
        let src_path = PathBuf::from(MODEL_STORED_PATH);
        let dest_path = PathBuf::from(MODEL_BACKUP_STORED_PATH);
        if src_path.exists() {
            let entries = read_dir(&src_path).expect("Failed to read directory!");
            for entry in entries {
                if let Ok(file) = entry {
                    let file_name = file.file_name().into_string().unwrap();
                    let model_src_path = src_path.join(&file_name);
                    let model_dest_path = dest_path.join(&file_name);
                    match copy(model_src_path, model_dest_path) {
                        Ok(_) => (),
                        Err(err) => return Err(err.to_string())
                    }
                }
            }
            return Ok(());
        }
        Err("Failed to backup models!".to_string())
    }));

    let _ = glob_daemon.start();
}