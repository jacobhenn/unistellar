use crate::{Args, State};

use std::fs::File;

use color_eyre::eyre::{Result, WrapErr};

use rocket::data::{Data, ToByteUnit};

use surrealdb::engine::remote::ws::Client;

use tracing::instrument;
use ulid::Ulid;

#[instrument(level = "debug", skip_all)]
pub async fn store_media(
    args: &Args,
    state: &rocket::State<State<Client>>,
    data: Data<'_>,
) -> Result<()> {
    let media_ulid = Ulid::new();

    let mut media_path = args.media_dir.clone();
    media_path.push(media_ulid.to_string());

    debug!("writing media to {media_path:?}");

    data.open(8.mebibytes())
        .into_file(media_path)
        .await
        .wrap_err("failed to write media data to file")?;

    Ok(())
}
