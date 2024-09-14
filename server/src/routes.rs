//! Defines API route handlers via Rocket

use super::{err::LogMapErr, structs::User, State};

use std::{error::Error, fmt::Display};

use color_eyre::eyre::{ErrReport, OptionExt, WrapErr};

use rocket::http::Status;

use surrealdb::engine::remote::ws::Client;

use tracing::instrument;

use uuid::Uuid;

/// Route "/user/<id>": data of user with a given user id. If a user with the given ID does
/// not exist, returns 404.
#[instrument(skip(state))]
#[get("/user/<id>")]
pub async fn users(state: &rocket::State<State<Client>>, id: Uuid) -> Result<String, Status> {
    // SAFETY: this does not allow for code injection as Rocket automatically validates the "<id>"
    // URL fragment as a UUID.
    let users: Vec<User> = state
        .db
        .query(format!("SELECT * FROM user WHERE user_id == '{id}'"))
        .await
        .and_then(|mut result| result.take(0))
        .log_map_err(|_| Status::InternalServerError)?;

    let user = users.first().ok_or(Status::NotFound)?;

    Ok(serde_json::to_string(user)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}
