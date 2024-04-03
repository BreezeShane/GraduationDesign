// cache.rs
pub const USER_PIC_PATH: &str = "./data_src/";
pub const DATASETS_DIRECTORY: &str = "./datasets/";

// doc_database.rs
pub const QUEUE_STORED_PATH: &str = "./queue.db";
pub const QUEUE_MAX_LENGTH: usize = 10;

pub const DATASETS_STORED_PATH: &str = "./datasets.db";

// authenticator.rs
pub const JWT_EXPIRATION: i64 = 3900;

// feedback.rs
pub const FEEDBACK_EXPIRATION: i64 = 86400;

// daemon.rs
pub const TIMER_DURATION: u64 = 3600;