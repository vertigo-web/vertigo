/// Opt-in: lets a [`Display`](std::fmt::Display) type of your own be rendered by vertigo -
/// as an attribute value, and as a text node in [`dom!`](crate::dom).
///
/// Neither [`AttrValue`](crate::AttrValue) nor [`EmbedDom`](crate::EmbedDom) converts from
/// every `T: Display`, because a blanket impl over a foreign trait collides with everything:
///
/// * on the attribute side it would claim the whole of `AttrValue`'s `From` space, so nothing
///   else could ever convert into an attribute - [`TwClass`](crate::TwClass) has to keep
///   hiding from `ToString` for exactly this reason - and a reactive wrapper that happens to
///   print would be silently flattened to a snapshot instead of subscribing;
/// * on the embedding side it would collide with `impl EmbedDom for MyComponent`, so a
///   printable type could never render real DOM instead of a text node.
///
/// Gating the same blanket on a trait of vertigo's own keeps the convenience without either
/// collision: nothing is rendered until it says so, and saying so is one line.
///
/// ```rust
/// use vertigo::{DomDisplay, dom};
///
/// enum Route {
///     Home,
///     Post(u32),
/// }
///
/// impl std::fmt::Display for Route {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Route::Home => write!(f, "/"),
///             Route::Post(id) => write!(f, "/post/{id}"),
///         }
///     }
/// }
///
/// impl DomDisplay for Route {}
///
/// let route = Route::Post(7);
/// let link = dom! { <a href={&route}>{&route}</a> };
/// ```
///
/// That single impl covers every form the value arrives in: `Route`, `&Route`,
/// [`Computed<Route>`](crate::Computed), [`Value<Route>`](crate::Value), the optional
/// variants of those, and references to any of them.
///
/// A type which renders real DOM implements [`EmbedDom`](crate::EmbedDom) instead, and must
/// not implement this - the two are alternatives, and asking for both is a coherence error.
///
/// The advice lives here rather than only on the traits which fail, because those see
/// `&Route` where this sees `Route` - and `impl DomDisplay for &Route {}` is not the fix.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not something vertigo renders",
    label = "no `DomDisplay` impl for `{Self}`",
    note = "if `{Self}` is yours and implements `Display`, one line opts it in: `impl vertigo::DomDisplay for {Self} {{}}`",
    note = "that covers `{Self}` and `&{Self}`, as an attribute value and as embedded text alike",
    note = "if `{Self}` comes from another crate, pass `value.to_string()` or wrap it in a newtype of your own - vertigo's `chrono` and `rust_decimal` features already cover those crates' date and decimal types"
)]
pub trait DomDisplay: std::fmt::Display {}

/// So one `impl DomDisplay for MyType {}` reaches `&MyType` as well, and a single blanket over
/// `T` serves both. Separate blankets over `T` and `&T` cannot: they overlap, because a
/// downstream crate may implement `DomDisplay` for a reference to a type of its own.
impl<T: DomDisplay + ?Sized> DomDisplay for &T {}

macro_rules! impl_dom_display {
    ($($typename:ty),* $(,)?) => {
        $(impl DomDisplay for $typename {})*
    };
}

// The string-ish types are absent on purpose. Each already has conversions that beat
// `to_string` at its own job - keeping a `&'static str` static, or an `Rc<String>` shared -
// and a marker impl would collide with them.
impl_dom_display!(
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
);

/// The types behind the `chrono` feature, which vertigo already knows how to put on the wire.
///
/// Each renders as its `Display` - `2026-08-30`, `2026-08-30 14:03:11`, and the same with an
/// offset for a `DateTime`. That is ISO 8601, not a localized date; a template wanting one
/// formats it itself, as it always had to.
///
/// The set matches the `JsJson` impls the feature already carries, plus `NaiveTime` and any
/// timezone rather than only `Utc`, both of which come free. `Weekday`, `Month` and
/// `TimeDelta` print names and ISO durations rather than values, so they stay out.
#[cfg(feature = "chrono")]
mod chrono_impls {
    use super::DomDisplay;

    impl DomDisplay for chrono::NaiveDate {}
    impl DomDisplay for chrono::NaiveTime {}
    impl DomDisplay for chrono::NaiveDateTime {}

    impl<Tz: chrono::TimeZone> DomDisplay for chrono::DateTime<Tz> where Tz::Offset: std::fmt::Display {}
}

/// `Decimal` behind the `rust_decimal` feature, for the same reason as the chrono types
/// above: the application cannot write this impl itself.
#[cfg(feature = "rust_decimal")]
mod rust_decimal_impls {
    use super::DomDisplay;

    impl DomDisplay for rust_decimal::Decimal {}
}
