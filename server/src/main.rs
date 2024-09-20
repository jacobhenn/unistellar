#[macro_use]
extern crate rocket;

use std::{
    fmt::Debug,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

use color_eyre::eyre::{bail, OptionExt, Result, WrapErr};

use dirs_next;

use err::LogMapErr;

use surrealdb::{
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
    Error, Surreal,
};

use tracing::{debug, info, instrument, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod db;
mod err;
mod routes;
mod structs;

const APP_NAME: &'static str = "unistellar-server";

/// A location to output logging.
#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
enum LogTo {
    /// Print logging to the console through stdout. This is the default in debug mode.
    Stdout,

    /// Write logging to a file (by default, $XDG_DATA_DIR/unistellar-server/logs) This is the
    /// default in release mode.
    File,
}

impl Default for LogTo {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Stdout
        } else {
            Self::File
        }
    }
}

/// UniStellar server.
#[derive(clap::Parser, Debug)]
struct Args {
    /// WebSocket address + port to connect to SurrealDB.
    #[arg(long)]
    db_addr: SocketAddr,

    /// Where to output logs. If absent, defaults to `stdout` if compiled in debug mode or `file`
    /// if compiled in release mode.
    #[arg(value_enum, long)]
    log_to: Option<LogTo>,
}

/// If the given path exists and is a directory, do nothing. If the given path does not exist,
/// attempt to create a new directory with that path.
///
/// # Errors
///
/// - The given path exists but is not a directory
/// - Failed to create a new directory with the given path (e.g. because a parent directory doesn't
/// exist)
#[instrument]
fn ensure_dir_exists(path: impl AsRef<Path> + Debug) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        fs::create_dir(&path).wrap_err_with(|| format!("failed to create directory {path:?}"))?;
    } else if !path.is_dir() {
        bail!("{path:?} exists but is not a directory");
    }

    Ok(())
}

/// Get a path to an assigned data directory for this application (cross-platform).
///
/// On Linux, returns "~/.local/share/unistellar-server".
#[instrument]
fn get_data_dir_path() -> Result<PathBuf> {
    let mut data_dir_path = dirs_next::data_dir().ok_or_eyre("could not locate data directory")?;
    data_dir_path.push(APP_NAME);

    ensure_dir_exists(&data_dir_path)?;

    Ok(data_dir_path)
}

/// Initialize asynchronous logging to the given destination type. The returned [`WorkerGuard`]
/// is an RAII guard which controls destruction of the resource handle and log queue used by the
/// worker thread, and it is returned so that it is only dropped when `main`'s scope ends.
///
/// If compiled in release mode, by default the logs will be written to files in the logging
/// directory timestamped to the instant logging was initialized - e.g. "~/.local/share/
/// unistellar-server/logs/2024-09-12T22:52:01.259739913+00:00.log"
///
/// Additionally, sets up span tracing to improve error messages.
#[instrument]
fn init_logging(log_to: LogTo) -> Result<WorkerGuard> {
    let (log_writer, guard) = match log_to {
        LogTo::Stdout => tracing_appender::non_blocking(std::io::stdout()),
        LogTo::File => {
            let mut log_file_path = get_data_dir_path()?;
            log_file_path.push("logs");

            ensure_dir_exists(&log_file_path)?;

            let now = chrono::Utc::now();
            log_file_path.push(format!("{}.log", now.to_rfc3339()));

            if log_file_path.exists() {
                panic!("timestamped log file path {log_file_path:?} already exists");
            }

            let log_file = fs::File::create(&log_file_path)
                .wrap_err_with(|| format!("failed to create log file at {log_file_path:?}"))?;

            tracing_appender::non_blocking(log_file)
        }
    };

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(log_to == LogTo::Stdout)
    //     .with_writer(log_writer)
    //     .with_filter(env_filter);

    // let subscriber = tracing_subscriber::Registry::default()
    //     .with(fmt_layer)
    //     .with(ErrorLayer::default());

    // tracing::subscriber::set_global_default(subscriber)
    //     .wrap_err("failed to set global logging subscriber")?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_ansi(log_to == LogTo::Stdout)
        .with_writer(log_writer)
        .init();

    info!("initialized logging");

    Ok(guard)
}

/// Shared server state available to all route handlers.
struct State<C: surrealdb::Connection> {
    /// A connection to the main database.
    db: Surreal<C>,
}

#[rocket::main]
async fn main() -> Result<()> {
    // install custom error handler to improve error messages
    color_eyre::install()?;

    // parse command-line arguments
    let args = Args::parse();

    let _guard = init_logging(args.log_to.unwrap_or_default())?;

    // connect to the database
    let db = db::connect(args.db_addr).await?;

    let state = State { db };

    info!("launching server");

    let _rocket = rocket::build()
        .manage(state)
        .mount(
            "/",
            routes![
                routes::user,
                routes::user_following,
                routes::user_followers,
                routes::uni_students,
                routes::course_search,
            ],
        )
        .launch()
        .await
        .wrap_err("server failure")?;

    Ok(())
}
