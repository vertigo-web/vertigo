use std::rc::Rc;

use crate::{DriverTrait, utils::EqBox};

pub type InstantType = u64;

#[derive(Clone)]
pub struct Instant {
    driver: EqBox<Rc<dyn DriverTrait>>,
    pub instant: InstantType,
}

impl Instant {
    pub fn new(driver: EqBox<Rc<dyn DriverTrait>>) -> Self {
        Self {
            instant: driver.now(),
            driver,
        }
    }

    pub fn now(&self) -> Self {
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
