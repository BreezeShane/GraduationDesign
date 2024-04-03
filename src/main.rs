pub mod cache;
pub mod config;
pub mod feedback;
pub mod user_manager;
pub mod doc_database;
pub mod model_manager;
pub mod authenticator;
pub mod training_show;


use std::sync::{Arc, Mutex};
//use tokio::sync::Mutex;
use authenticator::{handler_sign_in, handler_sign_out, handler_sign_up, middleware_authorize};
use cache::{handler_recieve_uploaded_dataset, handler_recieve_uploaded_pic};
use config::{DATASETS_STORED_PATH, QUEUE_STORED_PATH};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use feedback::handler_feedback;
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
        .route("/:user_id/upload_pic", post(handler_recieve_uploaded_pic))
        .route("/:user_id/feedback", post(handler_feedback))
    // .route("/admin/:user_id/", get())
        .route("/admin/xch_dset_stat", post(handler_xch_dset_stat))
    // .route("/admin/:user_id/feedback_manage", get())
        .route("/admin/:user_id/user_manage", post(handler_ban_or_unban_user))
    // .route("/admin/:user_id/training_effect", get())
    // .route("/admin/:user_id/training_panel", get())
        .route("/admin/:user_id/dataset_manage/:file_name", post(handler_recieve_uploaded_dataset))
    // .route("/admin/:user_id/model_backup", get())
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
}