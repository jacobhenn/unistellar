//! Defines API route handlers via Rocket

use crate::structs::{Course, Name, USId};

use super::{err::LogMapErr, structs::User, State};

use std::{borrow::Cow, error::Error, fmt::Display, str::FromStr};

use color_eyre::eyre::{ErrReport, OptionExt, WrapErr};

use rocket::{http::Status, request::FromParam, response::Responder};

use serde::{de::DeserializeOwned, Serialize};

use surrealdb::{
    engine::remote::ws::Client,
    opt::{IntoQuery, QueryResult},
    Surreal,
};

use tracing::{debug, instrument, trace};

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

/// Wrapper which guarantees query safety when being parsed from an URL. Specifically, its
/// implementation of FromParam validates that it consists only of ASCII alphanumeric and whitespace
/// characters.
#[derive(Debug)]
struct CleanStr<'a>(&'a str);

impl<'a> FromParam<'a> for CleanStr<'a> {
    type Error = ();

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        if !param
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        {
            return Err(());
        }

        Ok(Self(param))
    }
}

// -------------------------------------------------------------------------------------------------
// SAFETY: you will see me interpolate captured URL fragments into query strings in the following
// route handlers. As long as the type of the URL fragment has a restrictive syntax (e.g. a ULID),
// this does not allow for query injection as Rocket parses the fragments before the handler
// function is even called, and an error at that point will result in a 404.
// -------------------------------------------------------------------------------------------------

/// GET "/api/user/<id>": data of user with a given user ID. If a user with the given ID does
/// not exist, returns 404.
///
/// Example:
/// ```json
/// {
///   "id": "01J7YZ7MC3P44547KT11KHXGJV",
///   "name": {
///     "first": "Jacob",
///     "last": "Henn"
///   },
///   "username": "jacobhenn",
///   "university": "01J7YZ7MBVRK9B50WM6E6ZABJ0",
///   "major": "01J7YZ7MBXM1C8R69K8Y087K0C",
///   "grad_year": 2026
/// }
/// ```
#[instrument(skip(state))]
#[get("/user/<id_param>", rank = 1)]
pub async fn user(
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

/// GET "/api/user/<id>/following": list of IDs of users that the given user is following. If a user
/// with the given id does not exist, returns 404.
///
/// Example:
/// ```json
/// ["01J7YXMV1FSVAERRYEPR93NRX9","01J7YXMV1FZ94VHC13RCTRZM09"]
/// ```
#[instrument(skip(state))]
#[get("/user/<id_param>/following")]
pub async fn user_following(
    state: &rocket::State<State<Client>>,
    id_param @ UlidParam(id): UlidParam,
) -> Result<String, Status> {
    let query = format!("SELECT VALUE out FROM follows WHERE in=user:{id}");

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

/// GET "/api/user/<id>/followers": list of IDs of users that follow the given user. If a user with
/// the given id does not exist, returns 404.
///
/// Example:
/// ```json
/// ["01J7YXMV1FSVAERRYEPR93NRX9","01J7YXMV1FZ94VHC13RCTRZM09"]
/// ```
#[instrument(skip(state))]
#[get("/user/<id_param>/followers")]
pub async fn user_followers(
    state: &rocket::State<State<Client>>,
    id_param @ UlidParam(id): UlidParam,
) -> Result<String, Status> {
    let query = format!("SELECT VALUE in FROM follows WHERE out=user:{id}");

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

/// GET "/api/university/<id>/students": list of IDs of users who attend the given university. If
/// the given university ID does not exist, returns 404.
///
/// Example:
/// ```json
/// ["01J7YZ7MC3C49R19BHX6DTPGJ2","01J7YZ7MC3P44547KT11KHXGJV"]
/// ```
#[instrument(skip(state))]
#[get("/university/<id_param>/students")]
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

/// Calculates distance between strings `a` and `b` in a slightly cursed way: computes Levenshtein
/// distance and subtracts the absolute difference between the lengths
fn cursed_string_distance(a: &impl AsRef<[u8]>, b: &impl AsRef<[u8]>) -> i32 {
    let (a, b) = (a.as_ref(), b.as_ref());

    triple_accel::levenshtein_exp(a, b) as i32 - (a.len() as i32 - b.len() as i32).abs()
}

async fn search_table<T, Key>(
    db: &Surreal<Client>,
    query: &str,
    get_key: impl Fn(&T) -> Key,
    get_id: impl Fn(&T) -> USId,
) -> Result<Vec<USId>, Status>
where
    T: serde::de::DeserializeOwned,
    Key: Ord,
{
    let mut results: Vec<T> = db
        .query(query)
        .await
        .and_then(|mut resp| resp.take(0))
        .log_map_err(|_| Status::InternalServerError)?;

    results.sort_by_cached_key(|result| get_key(result));

    let mut result_ids: Vec<USId> = Vec::with_capacity(results.len());
    result_ids.extend(results.into_iter().map(|result| get_id(&result)));

    Ok(result_ids)
}

/// GET "/api/course/search/<search>": list of IDs of courses whose names match the given search string,
/// sorted in increasing order of [`cursed_string_distance`] with the search string.
///
/// Example:
/// ```json
/// ["01J7YZ7MC3C49R19BHX6DTPGJ2","01J7YZ7MC3P44547KT11KHXGJV"]
/// ```
#[instrument(skip(state))]
#[get("/course/search/<search_param>")]
pub async fn course_search(
    state: &rocket::State<State<Client>>,
    search_param @ CleanStr(search): CleanStr<'_>,
) -> Result<String, Status> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SearchResult {
        id: USId,
        name: String,
        code: String,
    }

    let query =
        format!("SELECT id, name, code FROM course WHERE name ~ '{search}' OR code ~ '{search}'");

    let search_results = search_table::<SearchResult, _>(
        &state.db,
        &query,
        |course| {
            [&course.name, &course.code]
                .into_iter()
                .map(|key| cursed_string_distance(key, &search))
                .min()
                .expect("iterator is not empty")
        },
        |course| course.id,
    )
    .await?;

    Ok(serde_json::to_string(&search_results)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}

/// GET "/api/user/search/<search>": list of IDs of users whose names match the given search string,
/// sorted in increasing order of [`cursed_string_distance`] with the search string.
///
/// Example:
/// ```json
/// ["01J7YZ7MC3C49R19BHX6DTPGJ2","01J7YZ7MC3P44547KT11KHXGJV"]
/// ```
#[instrument(skip(state))]
#[get("/user/search/<search_param>", rank = 2)]
pub async fn user_search(
    state: &rocket::State<State<Client>>,
    search_param @ CleanStr(search): CleanStr<'_>,
) -> Result<String, Status> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SearchResult {
        id: USId,
        username: String,
        name: Name,
    }

    let query = format!(
        "SELECT id, username, name FROM user 
            WHERE username ~ '{search}' OR (name.first + ' ' + name.last) ~ '{search}'"
    );

    debug!("query: `{query}`");

    let search_results = search_table::<SearchResult, _>(
        &state.db,
        &query,
        |user| {
            [&user.username, &user.name.first, &user.name.last]
                .into_iter()
                .map(|s| cursed_string_distance(s, &search))
                .min()
                .expect("iterator is non-empty")
        },
        |user| user.id,
    )
    .await?;

    Ok(serde_json::to_string(&search_results)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}

/// GET "/api/uni/search/<search>": list of IDs of universities whose names match the given search
/// string, sorted in increasing order of [`cursed_string_distance`] with the search string.
///
/// Example:
/// ```json
/// ["01J7YZ7MC3C49R19BHX6DTPGJ2","01J7YZ7MC3P44547KT11KHXGJV"]
/// ```
#[instrument(skip(state))]
#[get("/uni/search/<search_param>", rank = 2)]
pub async fn uni_search(
    state: &rocket::State<State<Client>>,
    search_param @ CleanStr(search): CleanStr<'_>,
) -> Result<String, Status> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SearchResult {
        id: USId,
        name: String,
    }

    let query = format!("SELECT id, name FROM university WHERE name ~ '{search}'");

    let search_results = search_table::<SearchResult, _>(
        &state.db,
        &query,
        |uni| cursed_string_distance(&uni.name, &search),
        |uni| uni.id,
    )
    .await?;

    Ok(serde_json::to_string(&search_results)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}

/// GET "/api/major/search/<search>": list of IDs of majors whose names match the given search
/// string, sorted in increasing order of [`cursed_string_distance`] with the search string.
///
/// Example:
/// ```json
/// ["01J7YZ7MC3C49R19BHX6DTPGJ2","01J7YZ7MC3P44547KT11KHXGJV"]
/// ```
#[instrument(skip(state))]
#[get("/major/search/<search_param>", rank = 2)]
pub async fn major_search(
    state: &rocket::State<State<Client>>,
    search_param @ CleanStr(search): CleanStr<'_>,
) -> Result<String, Status> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SearchResult {
        id: USId,
        name: String,
    }

    let query = format!("SELECT id, name FROM major WHERE name ~ '{search}'");

    let search_results = search_table::<SearchResult, _>(
        &state.db,
        &query,
        |major| cursed_string_distance(&major.name, &search),
        |major| major.id,
    )
    .await?;

    Ok(serde_json::to_string(&search_results)
        .wrap_err("failed to serialize response")
        .log_map_err(|_| Status::InternalServerError)?)
}
