//! Defines API route handlers via Rocket

use super::{err::LogMapErr, structs::User, State};

use std::{error::Error, fmt::Display, str::FromStr};

use color_eyre::eyre::{ErrReport, OptionExt, WrapErr};

use rocket::{http::Status, request::FromParam};

use surrealdb::engine::remote::ws::Client;

use tracing::instrument;

use ulid::Ulid;

/// Wrapper to implement automatic parsing of ULIDs from URL params. I wouldn't need to do this
/// if I was using UUIDs but SurrealDB works better with ULIDs since they don't contain special
/// characters and they're lexicographically sortable, and plus ULIDs are just more aesthetically
/// pleasing.
#[derive(Debug)]
struct UlidParam(Ulid);

impl<'a> FromParam<'a> for UlidParam {
    type Error = <Ulid as FromStr>::Err;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(UlidParam(param.parse()?))
    }
}

/// Route "/user/<id>": data of user with a given user id. If a user with the given ID does
/// not exist, returns 404.
#[instrument(skip(state))]
#[get("/user/<id_param>")]
pub async fn users(
    state: &rocket::State<State<Client>>,
    id_param @ UlidParam(id): UlidParam,
) -> Result<String, Status> {
    // SAFETY: this does not allow for code injection as Rocket automatically validates URL
    // fragments according to their FromParam implementation.
    let user: Option<User> = state
        .db
        .select(("user", id.to_string()))
        .await
        .log_map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(serde_json::to_string(&user)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}
