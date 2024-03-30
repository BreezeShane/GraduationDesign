use std::{fs::create_dir, path::Path};

use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    let mut cli = Client::connect("postgresql://postgres:postgres@localhost/InsectSys", NoTls)?;
    
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS Account (
            id              SERIAL,
            nick_name       VARCHAR NOT NULL,
            password_salt   VARCHAR NOT NULL,
            password_hash   VARCHAR NOT NULL,
            email           VARCHAR PRIMARY KEY NOT NULL,
            contribution    SMALLINT NOT NULL,
            available       BOOLEAN NOT NULL,
            permissions     SMALLINT NOT NULL
        );
    ")?;
    print!("Created Account Table!\n");

    // Create Trainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS TFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      TIMESTAMP NOT NULL,
            from_user_email    VARCHAR NOT NULL,
            time_out        TIMESTAMP NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR NOT NULL,
            acceptable      BOOLEAN NOT NULL
        );
    ")?;
    print!("Created TFeedback Table!\n");

    // Create Untrainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS UFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      TIMESTAMP NOT NULL,
            from_user_email    VARCHAR NOT NULL,
            pic_link        TEXT NOT NULL,
            acceptable      BOOLEAN NOT NULL
        );
    ")?;
    print!("Created UFeedback Table!\n");

    const USER_PIC_PATH: &str = "./data_src/";
    let path = Path::new(USER_PIC_PATH);
    if !path.exists() {
        match create_dir(&path) {
            Ok(_) => println!("Root Src Directory initialized."),
            Err(e) => println!("Error creating root directory: {}", e)
        }
    }

    Ok(())
}