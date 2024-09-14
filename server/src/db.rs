use std::net::SocketAddr;

use color_eyre::{eyre::WrapErr, Result};

use surrealdb::{
    engine::{
        local::{self, Db},
        remote::ws::{self, Client},
    },
    opt::auth::Root,
    Surreal,
};

use tracing::instrument;

const DB_ROOT_PASS: &'static str = "root";

/// Create and return a connection to a SurrealDB database at the given address and port.
#[instrument]
pub async fn connect(db_addr: SocketAddr) -> Result<Surreal<Client>> {
    info!("connecting to database");

    let db = Surreal::new::<ws::Ws>(db_addr)
        .await
        .wrap_err_with(|| format!("could not connect to database at {db_addr}"))?;

    info!("signing in to database");

    db.signin(Root {
        username: "root",
        password: DB_ROOT_PASS,
    })
    .await?;

    db.use_ns("unistellar").use_db("main").await?;

    Ok(db)
}

/// Create and return a connection to a temporary in-memory database, to be used in testing.
#[instrument]
pub async fn in_memory() -> Result<Surreal<Db>> {
    info!("creating in-memory database");

    let db = Surreal::new::<local::Mem>(())
        .await
        .wrap_err("failed to create in-memory database")?;

    Ok(db)
}
