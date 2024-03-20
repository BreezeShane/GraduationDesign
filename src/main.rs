use axum::{
    routing::get,
    Router,
};
use bb8::Pool; // PooledConnection
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// pub mod postgresql;
pub mod authenticator;
pub mod cache;
pub mod dataset_io;
pub mod feedback;
pub mod model_backup;
pub mod pic_io;
pub mod training_show;
pub mod user_manager;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "insects-identifier=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // set up connection pool
    let manager: PostgresConnectionManager<NoTls> =
        PostgresConnectionManager::new_from_stringlike("
        host=localhost user=postgres password=postgres dbname=InsectSys
        ", NoTls).unwrap();
    
    let pool: Pool<PostgresConnectionManager<NoTls>> = 
        Pool::builder().build(manager).await.unwrap();

    // build our application with a single route
    let app = Router::new() 
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sign_in", get(|| async { "Hello, World!" }))
        .route("/sign_up", get(|| async { "Hello, World!" }))
        .route("/sign_out", get(|| async { "Hello, World!" }))
        .route("/:user_id/result", get(|| async { "Hello, World!" }))
        .route("/:user_id/feedback", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/feedback_manage", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/user_manage", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/training_effect", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/training_panel", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/dataset_manage", get(|| async { "Hello, World!" }))
        .route("/:user_id/admin/model_backup", get(|| async { "Hello, World!" }))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}