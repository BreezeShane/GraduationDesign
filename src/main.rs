pub mod daemon;
pub mod config;
pub mod io_agent;
pub mod feedback;
pub mod user_manager;
pub mod doc_database;
pub mod model_manager;
pub mod authenticator;
pub mod dl_svc;

use std::{fs::copy, io, path::PathBuf, sync::{Arc, Mutex}};
//use tokio::sync::Mutex;
use authenticator::{handler_sign_in, handler_sign_up, middleware_authorize, handler_transfer_permission_to_role};
use dl_svc::handler_infer;
use chrono::Local;
use daemon::{Cronie, Daemon};
use io_agent::handler_upload_pic;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use feedback::{handler_acc_rej_fb, handler_fetch_trainable_fb, handler_fetch_ufb, handler_label_pic, handler_subm_fb};
use model_manager::handler_fetch_all_models;
use postgres::Client;
use std::fs::read_dir;
use tower_http::{trace::TraceLayer, cors::{CorsLayer, Any}};
use tokio_postgres::{Config, NoTls};
use axum::{
    body::Bytes, extract::{DefaultBodyLimit, FromRef, MatchedPath}, http::{HeaderMap, Method, Request}, middleware, response::Response, routing::{get, post}, Router
};
use user_manager::{handler_suspend_or_unsuspend_user, handler_user_info};
use doc_database::{
    DatasetVec, DatasetTrait,
    Queue, QueueTrait
};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{info, info_span, Level, Span};
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

use crate::{config::{MODEL_BACKUP_STORED_PATH, MODEL_STORED_PATH}, dl_svc::handler_authenticate_ssh, io_agent::handler_fetch_image, model_manager::handler_file_operation, user_manager::handler_fetch_all_users};

// use axum_macros::debug_handler; // Important!

#[derive(Clone, Debug)]
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

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%FT%T%.3f"))
    }
}


#[tokio::main]
async fn main() {
    // let file_appender = tracing_appender::rolling::daily("./tmp", "tracing.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true)
        .with_timer(LocalTimer);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        // .with_writer(non_blocking)
        // .with_ansi(false)
        .event_format(format)
        .init();
    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
    //             // axum logs rejections from built-in extractors with the `axum::rejection`
    //             // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
    //             "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
    //         }),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    let mut config = Config::new();
    config.host("localhost");
    config.user("postgres");
    config.password("postgres");
    config.dbname("insectsys");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(config, NoTls, mgr_config);

    let multi_state = MultiState {
        db_pool: Pool::builder(mgr).max_size(16).build().unwrap(),
        dset_db: Arc::new(
            Mutex::new(
                DatasetVec::load()
            )
        ),
        train_queue: Arc::new(
            Mutex::new(
                Queue::load()
            )
        )
    };
    // build our application with a single route

    let app = Router::new()
        .route("/user/info/:useremail", post(handler_user_info))
        .route("/user/check_role/:useremail", get(handler_transfer_permission_to_role))
        .route("/:useremail/upload_pic", post(handler_upload_pic))
        .route("/user/subm_fb", post(handler_subm_fb))
        .route("/user/infer", post(handler_infer))
        .route("/user/label_pic", get(handler_fetch_ufb).post(handler_label_pic))
        .route("/fetch_image", get(handler_fetch_image))

        .route("/admin/feedback_manage", get(handler_fetch_trainable_fb).post(handler_acc_rej_fb))
        .route("/admin/user_manage", get(handler_fetch_all_users).post(handler_suspend_or_unsuspend_user))
        .route("/admin/model_manage", get(handler_fetch_all_models).post(handler_file_operation))
        // .route("/admin/:user_id/dataset_manage/:file_name", post(handler_upload_dset))
        .route("/admin/authenticate_ssh/:useremail", post(handler_authenticate_ssh))
        .route_layer(middleware::from_fn(middleware_authorize))
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sign_in", post(handler_sign_in))
        .route("/sign_up", post(handler_sign_up))
        .with_state(multi_state)
        .layer(DefaultBodyLimit::max(4 * 1024 * 1024)) // 4 * 1024 * 1024 bytes
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {
                    tracing::debug!("The request body is {:#?}, during time {:#?}", _request, _span);
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    tracing::debug!("The response is {:#?}, during time {:#?}, the latency={:#?}", _response, _span, _latency);
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    tracing::debug!("The chunk is {:#?}, during time {:#?}, the latency={:#?}", _chunk, _span, _latency);
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        tracing::debug!("The trailers are {:#?}, during time {:#?}, the stream duration={:#?}", _trailers, _span, _stream_duration);
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::debug!("The error info is {:#?}, during time {:#?}, the latency={:#?}", _error, _span, _latency);
                    },
                ),
        ).layer(cors_layer);

    // run our app with hyper, listening globally on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("listening on http://{}", listener.local_addr().unwrap());
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
        let right_now = Local::now().timestamp();
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