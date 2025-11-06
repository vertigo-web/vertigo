use crate::driver_module::api::api_import;
use std::time::Duration;

/// Duration in seconds, returned from [Instant] methods.
pub type InstantType = u64;

/// Monotonically non-decreasing clock using a driver, similar to [std::time::Instant].
#[derive(Clone)]
pub struct Instant {
    pub instant: InstantType,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            instant: api_import().instant_now(),
        }
    }

    #[must_use]
    pub fn refresh(&self) -> Self {
        Self {
            instant: api_import().instant_now(),
        }
    }

    pub fn elapsed(&self) -> InstantType {
        api_import().instant_now() - self.instant
    }

    pub fn seconds_elapsed(&self) -> InstantType {
        self.elapsed() / 1000
    }

    pub fn add_duration(&self, time: Duration) -> Self {
        let new_instant = self.instant + time.as_millis() as u64;

        Self {
            instant: new_instant,
        }
    }

    pub fn is_expire(&self) -> bool {
        api_import().instant_now() > self.instant
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}
