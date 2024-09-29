//! Structure definitions that map onto the database schema.

use chrono::TimeDelta;

use serde_with::{serde_as, DurationSeconds};

use ulid::Ulid;

/// See [`USId`]
#[derive(serde::Deserialize, Debug, Clone, Copy)]
enum IdInner {
    String(Ulid),
}

/// UniStellar ID - basically just a (ULID)[https://github.com/ulid/spec]. This is a wrapper
/// to make it easier to deal with SurrealDB IDs since we know that everything is going to be
/// ULIDs.
///
/// Basically, SurrealDB is set up so that record IDs are arbitrary strings with namespace
/// specifiers, so their deserialized structure is quite nested and awkward to deal with. However,
/// we would like to use the record IDs for our user ids, university ids, etc. because it would
/// be even more awkward to have two different IDs for each thing. The solution I'm taking is to
/// make this helper struct with an asymmetric implementation of `Serialize` and `Deserialize` that
/// "forgets" all of the awkward structure of SurrealDB IDs when sending API responses, but still
/// correctly deserializes them from the results of database queries.
#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct USId {
    id: IdInner,
}

impl serde::Serialize for USId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let IdInner::String(id) = self.id;

        let s = format!("{}", id);
        serializer.serialize_str(&s)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Name {
    pub first: String,
    pub last: String,
}

/// An individual user of UniStellar.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct User {
    pub id: USId,
    pub name: Name,
    pub username: String,
    pub university: USId,
    pub major: USId,
    pub grad_year: i32,
}

/// A course; for now, these are independent of university since many courses are ubiquitous
/// and offered at every university, and it would be convenient for users to be able to search for
/// other users based on shared university and shared courses independently.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Course {
    pub id: USId,
    pub name: String,
}

/// A kind/state of an activity; e.g. "planning" or "completed"
#[serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ActivityData {
    /// The user is planning to do an activity
    Planning,

    /// The user has completed an activity
    Completed,

    /// The user has worked on an activity
    WorkedOn {
        /// The duration for which the user worked on the activity. Serialized as a number of
        /// seconds. This will appear as `duration_secs` in serialized JSON due to the rename
        /// attribute. This is for readability as JSON has no native duration type and I'm just
        /// sending it as a number of seconds.
        #[serde_as(as = "DurationSeconds<i64>")]
        #[serde(rename = "duration_secs")]
        duration: TimeDelta,
    },
}

#[serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Stats {
    /// The total number of assignments that the user has completed.
    pub assignments_completed: usize,

    /// The total duration that the user has worked on all activities.
    #[serde_as(as = "DurationSeconds<i64>")]
    #[serde(rename = "secs_worked")]
    pub duration_worked: TimeDelta,
}
