use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    let mut cli = Client::connect("postgresql://postgres:postgres@localhost/InsectSys", NoTls)?;
    
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS Account (
            id              SERIAL PRIMARY KEY,
            nick_name       VARCHAR NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR NOT NULL,
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
            from_user_id    SERIAL NOT NULL,
            time_out        TIMESTAMP NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR NOT NULL
        );
    ")?;
    print!("Created TFeedback Table!\n");

    // Create Untrainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS UFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      TIMESTAMP NOT NULL,
            from_user_id    SERIAL NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR
        );
    ")?;
    print!("Created UFeedback Table!\n");

    Ok(())
}