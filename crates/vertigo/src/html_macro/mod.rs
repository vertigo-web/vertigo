mod embed;

pub use embed::Embed;

/// Allows to define a Css styles factory function for virtual DOM.
///
/// ```rust
/// use vertigo::css_fn;
///
/// css_fn! { green_on_red, "
///     color: green;
///     background-color: red;
/// " }
/// ```
#[macro_export]
macro_rules! css_fn {
    ($name: ident, $block: tt) => {
        fn $name() -> vertigo::Css {
            $crate::css!($block)
        }
    };
}

/// Allows to define a Css styles factory function for virtual DOM
/// based on existing function but with added new rules.
///
/// ```rust
/// use vertigo::{css_fn, css_fn_push};
///
/// css_fn! { green, "
///     color: green;
/// " }
///
/// css_fn_push! { green_and_italic, green, "
///     font-style: italic;
/// " }
/// ```
#[macro_export]
macro_rules! css_fn_push {
    ($name: ident, $base: ident, $block: tt) => {
        fn $name() -> vertigo::Css {
            // TODO: Handle dynamic csses
            $base().push_str($crate::css_block!($block))
        }
    };
}

#[cfg(test)]
mod tests;
