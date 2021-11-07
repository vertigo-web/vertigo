use core::ops::{Try, FromResidual, ControlFlow};

#[derive(PartialEq, Clone, Debug)]
pub enum Resource<T: PartialEq> {
    Loading,
    Ready(T),
    Error(String),
}

pub enum ResourceError {
    Loading,
    Error(String),
}

impl<T: PartialEq> Try for Resource<T> {
    type Output = T;
    type Residual = ResourceError;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Resource::Ready(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<ResourceError, Self::Output> {
        match self {
            Self::Loading => ControlFlow::Break(ResourceError::Loading),
            Self::Error(message) => ControlFlow::Break(ResourceError::Error(message)),
            Self::Ready(value) => ControlFlow::Continue(value),
        }
    }
}


impl<T: PartialEq> FromResidual<ResourceError> for Resource<T> {
    fn from_residual(residual: ResourceError) -> Resource<T> {
        match residual {
            ResourceError::Error(message) => Resource::Error(message),
            ResourceError::Loading => Resource::Loading
        }
    }
}

impl<T: PartialEq> Resource<T> {
    pub fn map<K: PartialEq>(self, map: fn(T) -> K) -> Resource<K> {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(map(data)),
            Resource::Error(err) => Resource::Error(err),
        }
    }

    pub fn ref_map<K: PartialEq>(&self, map: fn(&T) -> K) -> Resource<K> {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(map(data)),
            Resource::Error(err) => Resource::Error(err.clone()),
        }
    }
}

impl<T: PartialEq + Clone> Resource<T> {
    pub fn ref_clone(&self) -> Resource<T> {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(data.clone()),
            Resource::Error(error) => Resource::Error(error.clone())
        }
    }
}

/*
Exampl:

fn get1() -> Resource<String> {
    todo!()
}

fn get2() -> Resource<u32> {
    todo!()
}

fn get3() -> Resource<String> {
    let val1: String = get1()?;
    let val2: u32 = get2()?;

    todo!();
}
*/
