// use deadpool_postgres::{Manager, Pool};
// use tokio_postgres::{Config, NoTls};
// use core::panic;

// pub fn connect_db() -> Result<Client, Error>{
//     Client::connect("postgresql://postgres:postgres@localhost/InsectSys", NoTls)
// }

// pub fn retry_connect_db() -> Result<(), E> {
//     let mut client;
//     for i in 1..=3 {
//         client = Client::connect("postgresql://postgres:postgres@localhost/InsectSys", NoTls)?;

//     }
// }

// pub fn init_tables() -> Result<(), Error> {
//     let mut cli = match connect_db() {
//         Ok(client) => client,
//         Err(error) => panic!("Failed to connect database! {:?}", error),
//     };
    
//     // Create User Table.
//     cli.batch_execute("
//         CREATE TABLE IF NOT EXISTS User (
//             id              SERIAL PRIMARY KEY,
//             nick_name       VARCHAR NOT NULL,
//             email           VARCHAR NOT NULL,
//             contribution    SMALLINT NOT NULL,
//             available       BOOLEAN NOT NULL
//         )
//     ")?;

//     // Create Trainable Feedback Table.
//     cli.batch_execute("
//         CREATE TABLE IF NOT EXISTS TFeedback (
//             id              SERIAL PRIMARY KEY,
//             time_stamp      TIMESTAMP NOT NULL,
//             from_user_id    SERIAL NOT NULL,
//             time_out        TIMESTAMP NOT NULL,
//             pic_link        TEXT NOT NULL,
//             real_label      VARCHAR NOT NULL,
//         )
//     ")?;

//     // Create Untrainable Feedback Table.
//     cli.batch_execute("
//         CREATE TABLE IF NOT EXISTS UFeedback (
//             id              SERIAL PRIMARY KEY,
//             time_stamp      TIMESTAMP NOT NULL,
//             from_user_id    SERIAL NOT NULL,
//             pic_link        TEXT NOT NULL,
//             real_label      VARCHAR,
//         )
//     ")?;
    
//     Ok(())
// }