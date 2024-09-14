//! Structure definitions that map onto the database schema.

use serde::{Deserialize, Serialize};

use surrealdb::sql::{Id, Thing};

use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

/// An individual user of UniStellar.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct User {
    /// Unique user ID. This is separate from the user's _record id_ in SurrealDB.
    // TODO: possibly figure out a way to deal with the record ids nicer so this isn't necessary.
    pub user_id: Uuid,
    pub name: Name,
    pub university: Uuid,
    pub grad_year: i32,
}
