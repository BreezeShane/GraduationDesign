pub mod authenticator;
pub mod cache;
pub mod feedback;
pub mod model_backup;
pub mod training_show;
pub mod user_manager;

use authenticator::{handler_sign_in, handler_sign_out, handler_sign_up, middleware_authorize};
use cache::handler_recieve_uploaded_pic;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use feedback::handler_feedback;
use tokio_postgres::{Config, NoTls};
use axum::{
    extract::DefaultBodyLimit, middleware, routing::{get, post}, Router
};
use user_manager::handler_ban_or_unban_user;


#[tokio::main]
async fn main() {
    let mut config = Config::new();
    config.host("localhost");
    config.user("postgres");//数据库用户名
    config.password("postgres");//数据库密码
    config.dbname("InsectSys");//数据库名称
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(config, NoTls, mgr_config);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();

    // build our application with a single route
    
    let app = Router::new() 
    // .route("/:user_id/result", get())
        .route("/:user_id/upload_pic", post(handler_recieve_uploaded_pic))
        .route("/:user_id/feedback", post(handler_feedback))
    // .route("/:user_id/admin/", get())
    // .route("/:user_id/admin/feedback_manage", get())
        // .route("/:user_id/admin/user_manage", get())
        .route("/:user_id/admin/user_manage", post(handler_ban_or_unban_user))
    // .route("/:user_id/admin/training_effect", get())
    // .route("/:user_id/admin/training_panel", get())
    // .route("/:user_id/admin/dataset_manage", get())
    // .route("/:user_id/admin/model_backup", get())
        .route("/sign_out/:user_id", get(handler_sign_out))
        .route_layer(middleware::from_fn(middleware_authorize))
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sign_in", post(handler_sign_in))
        .route("/sign_up", post(handler_sign_up))
        .with_state(pool)
        .layer(DefaultBodyLimit::max(1024));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}