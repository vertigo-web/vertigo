use crate::{Instant, Resource};
use std::rc::Rc;

#[derive(PartialEq)]
pub enum ApiResponse<T> {
    Uninitialized,
    Data {
        value: Resource<Rc<T>>,
        expiry: Option<Instant>,
    },
}

impl<T> ApiResponse<T> {
    pub fn new(value: Resource<Rc<T>>, expiry: Option<Instant>) -> Self {
        Self::Data { value, expiry }
    }

    pub fn new_loading() -> Self {
        ApiResponse::Data {
            value: Resource::Loading,
            expiry: None,
        }
    }

    pub fn get_value(&self) -> Resource<Rc<T>> {
        match self {
            Self::Uninitialized => Resource::Loading,
            Self::Data { value, expiry: _ } => value.clone(),
        }
    }

    pub fn needs_update(&self) -> bool {
        match self {
            ApiResponse::Uninitialized => true,
            ApiResponse::Data { value: _, expiry } => {
                let Some(expiry) = expiry else {
                    return false;
                };

                expiry.is_expire()
            }
        }
    }
}

impl<T> Clone for ApiResponse<T> {
    fn clone(&self) -> Self {
        match self {
            ApiResponse::Uninitialized => ApiResponse::Uninitialized,
            ApiResponse::Data { value, expiry } => ApiResponse::Data {
                value: value.clone(),
                expiry: expiry.clone(),
            },
        }
    }
}
