// io_agent.rs
pub const USER_PIC_PATH: &str = "./data_src/";
pub const DATASETS_DIRECTORY: &str = "./datasets/";
pub const TFEEDBACK_STORED_DIRECTORY: &str = "./tfeedback/";
pub const UFEEDBACK_STORED_DIRECTORY: &str = "./ufeedback/";
pub const DATA_TO_TRAIN_DIRECTORY: &str = "./data2train/";

// doc_database.rs
pub const QUEUE_STORED_PATH: &str = "./queue.db";
pub const QUEUE_MAX_LENGTH: usize = 10;

pub const DATASETS_STORED_PATH: &str = "./datasets.db";

// authenticator.rs
pub const JWT_EXPIRATION: i64 = 3600 + 300; // 1h + 5min
pub const JWT_REFRESH_PERIOD: i64 = 600;

// feedback.rs
pub const FEEDBACK_EXPIRATION: i64 = 3600 * 24 * 7; // 7 days

// daemon.rs
pub const TIMER_DURATION: u64 = 3600; // 1h

// main.rs + model_manage.rs
pub const MODEL_STORED_PATH: &str = "./models/";
pub const COMPILED_MODEL_STORED_PATH: &str = "./models/compiled/";
pub const MODEL_BACKUP_STORED_PATH: &str = "./.modbak/";

// dl_svc.rs
pub const DL_SVC_HOST: &str = "https://localhost:8182";
