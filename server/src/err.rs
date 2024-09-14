//! Helpers for improving error handling

use std::fmt::Display;

/// Result extension trait for inspecting an error with a logging function. This allows for a very
/// terse syntax for the operation of logging an error while continuing to propagate it.
pub trait LogMapErr: Sized {
    type Ok;
    type Err;

    fn log_map_err<E>(self, f: impl Fn(Self::Err) -> E) -> Result<Self::Ok, E>;

    fn log_err(self) -> Result<Self::Ok, Self::Err> {
        self.log_map_err(|e| e)
    }
}

impl<T, E> LogMapErr for Result<T, E>
where
    E: Display,
{
    type Ok = T;
    type Err = E;

    fn log_map_err<E2>(self, f: impl Fn(E) -> E2) -> Result<T, E2> {
        self.map_err(|err| {
            tracing::error!("{err}");
            f(err)
        })
    }
}
