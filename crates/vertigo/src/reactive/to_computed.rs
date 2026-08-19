use super::Computed;

/// Convert the type into a [`Computed`].
pub trait ToComputed<T: Clone + PartialEq> {
    fn to_computed(&self) -> Computed<T>;
}

macro_rules! impl_to_computed {
    ($typename: ty) => {
        impl ToComputed<$typename> for $typename {
            fn to_computed(&self) -> Computed<$typename> {
                let value = *self;
                Computed::from(move |_| value)
            }
        }
    };
}

impl_to_computed!(i8);
impl_to_computed!(i16);
impl_to_computed!(i32);
impl_to_computed!(i64);
impl_to_computed!(i128);
impl_to_computed!(isize);

impl_to_computed!(u8);
impl_to_computed!(u16);
impl_to_computed!(u32);
impl_to_computed!(u64);
impl_to_computed!(u128);
impl_to_computed!(usize);

impl_to_computed!(f32);
impl_to_computed!(f64);

impl_to_computed!(char);

impl_to_computed!(bool);

impl_to_computed!(());
