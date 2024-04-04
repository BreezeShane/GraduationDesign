pub mod daemon;
pub mod config;
pub mod io_cache;
pub mod feedback;
pub mod user_manager;
pub mod doc_database;
pub mod model_manager;
pub mod authenticator;
pub mod training_show;


use std::sync::{Arc, Mutex};
//use tokio::sync::Mutex;
use authenticator::{handler_sign_in, handler_sign_out, handler_sign_up, middleware_authorize};
use daemon::{Cronie, Daemon, Task};
use io_cache::{handler_upload_dset, handler_upload_pic};
use config::{DATASETS_STORED_PATH, QUEUE_STORED_PATH};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use feedback::{handler_acc_rej_fb, handler_fetch_all_fb, handler_fetch_ufb, handler_label_pic, handler_subm_fb};
use model_manager::handler_xch_dset_stat;
use tokio_postgres::{Config, NoTls};
use axum::{
    extract::{DefaultBodyLimit, FromRef}, middleware, routing::{get, post}, Router
};
use user_manager::handler_ban_or_unban_user;
use doc_database::{
    DatasetVec, DatasetTrait,
    Queue, QueueTrait
};

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
        .route("/:user_id/label_pic", get(handler_fetch_ufb).post(handler_label_pic))
        .route("/:user_id/subm_fb", post(handler_subm_fb))

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

    let feedback_manage_task = Task::new("auto_acc_fd", || {
        todo!()
    });
    let model_backup_task = Task::new("auto_bak_mod", || {
        todo!()
    });

    let mut glob_daemon = Daemon::new();
    let _ = glob_daemon.append_task(feedback_manage_task);
    let _ = glob_daemon.append_task(model_backup_task);

    let _ = glob_daemon.start();
}