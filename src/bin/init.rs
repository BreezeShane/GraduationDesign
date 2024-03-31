use std::{fs::{create_dir, File}, io::Write, path::Path};

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

    // init data source folder.
    const USER_PIC_PATH: &str = "./data_src/";
    let src_path = Path::new(USER_PIC_PATH);
    if !src_path.exists() {
        match create_dir(&src_path) {
            Ok(_) => println!("Root Src Directory initialized."),
            Err(e) => println!("Error creating root directory: {}", e)
        }
    }

    // init document database storage file
    const DOC_STORED_PATH: &str = "./doc.db";
    let doc_path = Path::new(DOC_STORED_PATH);
    let doc_path_display = doc_path.display();
    if !doc_path.exists() {
        let mut file = match File::create(&doc_path) {
            Err(err) => panic!("couldn't create {}: {:?}", doc_path_display, err),
            Ok(file) => file,
        };

        match file.write_all("".as_bytes()) {
            Err(err) => {
                panic!("couldn't write to {}: {:?}", doc_path_display, err)
            },
            Ok(_) => println!("successfully wrote to {}", doc_path_display),
        }
    }
    Ok(())
}