//! Support shared by the browser test binaries.

use std::error::Error;
use std::fmt::Display;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// Attaches a description of the step to a failure.
///
/// These tests used to say what went wrong through `expect` messages. Propagating with `?`
/// instead would leave only the bare driver error - `no such element`, with no hint which of a
/// thousand lines produced it - so the message is kept and prefixed onto the error.
pub trait Ctx<T> {
    fn ctx(self, what: impl Display) -> TestResult<T>;
}

impl<T, E: Display> Ctx<T> for Result<T, E> {
    fn ctx(self, what: impl Display) -> TestResult<T> {
        self.map_err(|err| format!("{what}: {err}").into())
    }
}

impl<T> Ctx<T> for Option<T> {
    fn ctx(self, what: impl Display) -> TestResult<T> {
        self.ok_or_else(|| what.to_string().into())
    }
}
