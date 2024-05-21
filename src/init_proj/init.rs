use std::{fs::{create_dir, File}, io::Write, path::Path};

use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    let mut cli = Client::connect("postgresql://postgres:postgres@localhost/insectsys", NoTls)?;

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
            time_stamp      BIGINT NOT NULL,
            from_user_email VARCHAR NOT NULL,
            time_out        BIGINT NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR NOT NULL,
            submit_count    BIGINT NOT NULL
        );
    ")?;
    print!("Created TFeedback Table!\n");

    // Create Untrainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS UFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      BIGINT NOT NULL,
            from_user_email VARCHAR NOT NULL,
            pic_link        TEXT NOT NULL
        );
    ")?;
    print!("Created UFeedback Table!\n");

    // init data source folder.
    const USER_PIC_PATH: &str = "./data_src/";
    const DATASETS_DIRECTORY: &str = "./datasets/";
    const MODEL_STORED_PATH: &str = "./models/";
    const MODEL_BACKUP_STORED_PATH: &str = "./.modbak/";
    const TFEEDBACK_STORED_DIRECTORY: &str = "./tfeedback/";
    const UFEEDBACK_STORED_DIRECTORY: &str = "./ufeedback/";
    const DATA_TO_TRAIN_DIRECTORY: &str = "./data2train/";
    const COMPILED_MODEL_STORED_PATH: &str = "./models/compiled/";
    let vec_path = vec![
        USER_PIC_PATH,
        DATASETS_DIRECTORY,
        MODEL_STORED_PATH,
        MODEL_BACKUP_STORED_PATH,
        TFEEDBACK_STORED_DIRECTORY,
        UFEEDBACK_STORED_DIRECTORY,
        DATA_TO_TRAIN_DIRECTORY,
        COMPILED_MODEL_STORED_PATH,
    ];
    init_dirs(vec_path);

    // init document database storage file
    const QUEUE_STORED_PATH: &str = "./queue.db";
    touch_file(QUEUE_STORED_PATH);
    const DATASETS_STORED_PATH: &str = "./datasets.db";
    touch_file(DATASETS_STORED_PATH);

    Ok(())
}

fn init_dirs(vec_path: Vec<&str>) {
    let iter = vec_path.iter();

    for path in iter {
        let src_path = Path::new(path);
        if !src_path.exists() {
            match create_dir(src_path) {
                Ok(_) => println!("Root Src Directory initialized."),
                Err(e) => println!("Error creating root directory: {}", e)
            }
        }
    }
}

fn touch_file(path: &str) {
    let doc_path = Path::new(path);
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
}