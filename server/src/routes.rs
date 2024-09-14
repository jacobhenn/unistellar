//! Defines API route handlers via Rocket

use crate::structs::USId;

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

// -------------------------------------------------------------------------------------------------
// SAFETY: you will see me interpolate captured URL fragments into query strings in the following
// route handlers. As long as the type of the URL fragment has a restrictive syntax (e.g. a ULID),
// this does not allow for query injection as Rocket parses the fragments before the handler
// function is even called, and an error at that point will result in a 404.
// -------------------------------------------------------------------------------------------------

/// Route "/user/<id>": data of user with a given user id. If a user with the given ID does
/// not exist, returns 404.
#[instrument(skip(state))]
#[get("/user/<id_param>")]
pub async fn users(
    state: &rocket::State<State<Client>>,
    id_param @ UlidParam(id): UlidParam,
) -> Result<String, Status> {
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

/// Route "/uni/<id>/students": list of student IDs which attend the given university. If the given
/// university ID does not exist, returns 404.
#[instrument(skip(state))]
#[get("/uni/<id_param>/students")]
pub async fn uni_students(
    state: &rocket::State<State<Client>>,
    id_param @ UlidParam(id): UlidParam,
) -> Result<String, Status> {
    let query = format!("SELECT VALUE id FROM user WHERE university == university:{id}");

    let user_ids: Vec<USId> = state
        .db
        .query(query)
        .await
        .and_then(|mut resp| resp.take(0))
        .log_map_err(|_| Status::InternalServerError)?;

    Ok(serde_json::to_string(&user_ids)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}
