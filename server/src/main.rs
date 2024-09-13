#[macro_use]
extern crate rocket;

use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};

use dirs_next;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Error, Surreal,
};

use tracing::{debug, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::util::SubscriberInitExt;

const APP_NAME: &'static str = "unistellar";

const DB_ROOT_PASS: &'static str = include_str!(".db-root-pass");

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

async fn db_connect() -> Result<Surreal<Client>> {
    let db = Surreal::new::<Ws>("localhost::8000")
        .await
        .context("could not connect to the database")?;

    db.signin(Root {
        username: "root",
        password: DB_ROOT_PASS,
    })
    .await?;

    Ok(db)
}

fn get_data_dir() -> Result<PathBuf> {
    let mut data_dir_path = dirs_next::data_dir().context("could not locate data directory")?;
    data_dir_path.push(APP_NAME);

    if !data_dir_path.exists() {
        fs::create_dir(&data_dir_path)?;
    } else if !data_dir_path.is_dir() {
        bail!(
            "path to {APP_NAME} data directory ({data_dir_path:?}) exists but is not a directory"
        );
    }

    Ok(data_dir_path)
}

fn init_logging() -> Result<WorkerGuard> {
    let mut log_file_path = get_data_dir()?;

    let now = chrono::Utc::now();
    log_file_path.push(format!("{}.log", now.to_rfc3339()));

    if log_file_path.exists() {
        bail!("timestamped log file path {log_file_path:?} already exists; this is weird");
    }

    let log_file = fs::File::create(log_file_path).context("failed to create log file")?;

    let (log_writer, guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(log_writer)
        .init();

    info!("initialized logging");

    Ok(guard)
}

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let _guard = init_logging()?;

    let _rocket = rocket::build()
        .mount("/", routes![hello])
        .launch()
        .await
        .context("could not launch server")?;

    Ok(())
}
