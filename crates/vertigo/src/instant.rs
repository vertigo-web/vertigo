use crate::driver_module::driver_browser::DriverBrowser;

/// Duration in seconds, returned from [Instant] methods.
pub type InstantType = u64;

/// Monotonically nondecrasing clock using a driver, similar to [std::time::Instant].
#[derive(Clone)]
pub struct Instant {
    driver: DriverBrowser,
    pub instant: InstantType,
}

impl Instant {
    pub fn now(driver: DriverBrowser) -> Self {
        Self {
            instant: driver.now(),
            driver,
        }
    }

    #[must_use]
    pub fn refresh(&self) -> Self {
        Self {
            instant: self.driver.now(),
            driver: self.driver.clone(),
        }
    }

    pub fn elapsed(&self) -> InstantType {
        self.driver.now() - self.instant
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
