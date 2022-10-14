use std::rc::Rc;

use crate::{ApiImport};

/// Duration in seconds, returned from [Instant] methods.
pub type InstantType = u64;

/// Monotonically non-decreasing clock using a driver, similar to [std::time::Instant].
#[derive(Clone)]
pub struct Instant {
    api: Rc<ApiImport>,
    pub instant: InstantType,
}

impl Instant {
    pub fn now(api: Rc<ApiImport>) -> Self {
        Self {
            instant: api.instant_now(),
            api,
        }
    }

    #[must_use]
    pub fn refresh(&self) -> Self {
        Self {
            instant: self.api.instant_now(),
            api: self.api.clone(),
        }
    }

    pub fn elapsed(&self) -> InstantType {
        self.api.instant_now() - self.instant
    }

    pub fn seconds_elapsed(&self) -> InstantType {
        self.elapsed() / 1000
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}
