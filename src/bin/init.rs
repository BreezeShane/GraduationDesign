use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    let mut cli = Client::connect("postgresql://postgres:postgres@localhost/library", NoTls)?;
    
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS User (
            id              SERIAL PRIMARY KEY,
            nick_name       VARCHAR NOT NULL,
            email           VARCHAR NOT NULL,
            contribution    SMALLINT NOT NULL,
            available       BOOLEAN NOT NULL
        )
    ")?;

    // Create Trainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS TFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      TIMESTAMP NOT NULL,
            from_user_id    SERIAL NOT NULL,
            time_out        TIMESTAMP NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR NOT NULL,
        )
    ")?;

    // Create Untrainable Feedback Table.
    cli.batch_execute("
        CREATE TABLE IF NOT EXISTS UFeedback (
            id              SERIAL PRIMARY KEY,
            time_stamp      TIMESTAMP NOT NULL,
            from_user_id    SERIAL NOT NULL,
            pic_link        TEXT NOT NULL,
            real_label      VARCHAR,
        )
    ")?;
    Ok(())
}