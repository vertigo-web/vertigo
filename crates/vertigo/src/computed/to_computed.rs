use super::Computed;

/// A trait allowing converting the type into computed.
///
/// ```rust
/// use vertigo::{ToComputed, transaction};
///
/// let comp_1 = 5.to_computed();
/// let comp_2 = 'x'.to_computed();
/// let comp_3 = false.to_computed();
///
/// transaction(|context| {
///     assert_eq!(comp_1.get(context), 5);
///     assert_eq!(comp_2.get(context), 'x');
///     assert_eq!(comp_3.get(context), false);
/// });
///
/// ```
pub trait ToComputed<T: Clone> {
    fn to_computed(&self) -> Computed<T>;
}

impl<T: Clone + 'static> ToComputed<T> for Computed<T> {
    fn to_computed(&self) -> Computed<T> {
        self.clone()
    }
}

impl<T: Clone + 'static> ToComputed<T> for &Computed<T> {
    fn to_computed(&self) -> Computed<T> {
        (*self).clone()
    }
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
