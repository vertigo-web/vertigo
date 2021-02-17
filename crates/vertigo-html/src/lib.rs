mod embed;

pub use embed::Embed;

// Proc-macros can't be defined in the same crate, so all we can do is re-export it here from separate "sub-crate"
pub use vertigo_html_macro::{html, css, css_block};

// For convenience
pub use vertigo::VDomElement;

#[macro_export]
macro_rules! css_fn {
    ($name: ident, $block: tt) => {
        fn $name() -> vertigo::Css {
            $crate::css! ($block)
        }
    };
}

#[macro_export]
macro_rules! css_fn_push {
    ($name: ident, $base: ident, $block: tt) => {
        fn $name() -> vertigo::Css {
            $base().push($crate::css_block! ($block))
        }
    }
}

#[cfg(test)]
mod tests;
