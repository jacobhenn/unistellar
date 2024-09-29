use std::{
    ffi::OsStr,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::Parser;

use color_eyre::eyre::{self, WrapErr};

/// Run certain commands for setting up the server
#[derive(clap::Parser, Debug)]
struct Args {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    /// Start the Rust server
    RunServer,

    /// Start the SurrealDB database
    RunDb,

    /// Start a Surql interface attached to the running database
    Surql,

    /// Import a SurrealQL file into the database
    Import {
        /// File of queries to import. Must be a `.surql` file.
        file: PathBuf,
    },

    /// Initialize schemas and event hooks in the table without clearing old data or loading test data
    SetupTables,

    /// Clear the database and re-insert the test data in `surql/test_data.surql`
    ResetData,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Config {
    db_addr: String,
    db_store_path: Option<PathBuf>,
}

impl Config {
    fn db_url(&self) -> String {
        format!("http://{}", self.db_addr)
    }

    fn db_store_url(&self) -> Option<String> {
        self.db_store_path
            .as_ref()
            .map(|path| format!("rocksdb://{}", path.to_string_lossy()))
    }
}

macro_rules! run_cmd {
    ($cmd:expr, $($args:expr),*) => {
        {
            let mut cmd = std::process::Command::new($cmd);

            $(cmd.args($args);)*

            cmd.status().wrap_err("failed to spawn child").map(|_| ())
        }
    }
}

fn import_file(config: &Config, path: impl AsRef<OsStr>) -> eyre::Result<()> {
    run_cmd!(
        "surreal",
        ["import", "--conn", &config.db_url()],
        ["--ns", "unistellar", "--db", "main"],
        [path],
        ["-u", "root", "-p", "root"]
    )
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string("unistellar-helper.toml").wrap_err(
        format!("failed to read config file at 'unistellar-helper.toml"),
    )?)
    .wrap_err("failed to parse config")?;

    match args.subcommand {
        Subcommand::RunServer => run_cmd!("cargo", ["run", "--", "--db-addr", &config.db_addr])?,
        Subcommand::RunDb => {
            run_cmd!(
                "surreal",
                ["start"],
                config.db_store_url(),
                ["-A", "-b", &config.db_addr]
            )?;
        }
        Subcommand::Surql => run_cmd!(
            "surreal",
            ["sql", "--endpoint", &config.db_url(), "--pretty"],
            ["--ns", "unistellar", "--db", "main"],
            ["-u", "root", "-p", "root"]
        )?,
        Subcommand::Import { file } => import_file(&config, &file)?,
        Subcommand::SetupTables => import_file(&config, "surql/setup_tables.surql")?,
        Subcommand::ResetData => {
            for file_path in [
                "surql/clear_all.surql",
                "surql/setup_tables.surql",
                "surql/test_data.surql",
            ] {
                import_file(&config, file_path)?;
            }
        }
    }

    Ok(())
}
