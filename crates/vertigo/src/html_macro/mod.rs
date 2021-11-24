mod embed;

pub use embed::Embed;

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
            // TODO: Handle dynamic csses
            $base().push_str($crate::css_block! ($block))
        }
    }
}

#[cfg(test)]
mod tests;
