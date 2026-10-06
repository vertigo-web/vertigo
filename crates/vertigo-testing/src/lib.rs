//! Harness for end-to-end browser tests of vertigo apps.
//!
//! A test crate builds its apps, starts what they run against and describes that as its own
//! environment: a type implementing [`Env`] that dereferences to [`Core`]. Each test binary
//! (`harness = false`) is one suite, run by [`run_suite`]: it starts the environment once,
//! lazily, gives every browser test its own Chrome session wrapped in [`Ctx`], and checks the
//! browser console after each of them.
//!
//! The building blocks of an environment:
//!
//! - [`build`]: the apps, with the vertigo-cli library, and the backend, with cargo,
//! - [`serve`]: a build served in this process, as `vertigo serve` does,
//! - [`free_port`]: a port for a server the suite starts.
//!
//! Settings come from `E2E_*` environment variables, see [`Settings`].

pub mod build;
pub mod ctx;
pub mod env;
pub mod runner;
pub mod serve;

pub use ctx::{Ctx, HydrationReport, normalize, xpath_literal};
pub use env::{ChromeConfig, Core, Env, POLL, Settings, WAIT, free_port};
pub use runner::{Body, TestCase, TestFuture, known_failures, run_suite};

pub use reqwest;
pub use thirtyfour;

/// What the tests and the helpers of a test crate usually need.
pub mod prelude {
    pub use std::{sync::Arc, time::Duration};

    pub use anyhow::{Context, Result, anyhow, bail, ensure};
    pub use serde_json::{Value, json};
    pub use thirtyfour::{By, Key, WebElement, components::SelectElement};

    pub use crate::{
        browser_tests,
        ctx::{normalize, xpath_literal},
        http_tests,
        runner::{TestCase, known_failures, run_suite},
    };
}
