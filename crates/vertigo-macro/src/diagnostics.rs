//! Errors and warnings the macros report while they expand.
//!
//! A macro reports any number of errors with [`emit_error!`] and goes on, so the user gets
//! them all at once. [`expand`] runs the macro, and if it reported any, they replace what it
//! made: `compile_error!` invocations pointing at the spans given.
//!
//! [`emit_warning!`] warns only on a compiler with `proc_macro::Diagnostic` (nightly, see
//! `vertigo_macro_warnings` in build.rs) and is dropped elsewhere: rustc has no stable way for
//! a macro to warn.

use std::cell::RefCell;

use proc_macro2::{Span, TokenStream as TokenStream2};

thread_local! {
    static ERRORS: RefCell<Vec<syn::Error>> = const { RefCell::new(Vec::new()) };
}

/// The message of a diagnostic: `format!` arguments, or one value to display.
macro_rules! diagnostic_message {
    ($format:expr, $($argument:expr),+ $(,)?) => {
        format!($format, $($argument),+)
    };
    ($message:expr $(,)?) => {
        ::std::string::ToString::to_string(&$message)
    };
}

/// Report an error at a span (`proc_macro::Span` or `proc_macro2::Span`), with a message as
/// `format!` takes it, or one value to display.
macro_rules! emit_error {
    ($span:expr, $($message:tt)+) => {
        $crate::diagnostics::error($span, diagnostic_message!($($message)+))
    };
}

/// [`emit_error!`] at the macro's call site.
macro_rules! emit_call_site_error {
    ($($message:tt)+) => {
        emit_error!(::proc_macro2::Span::call_site(), $($message)+)
    };
}

/// Report a warning, as [`emit_error!`] takes it - on nightly only, see the module docs.
macro_rules! emit_warning {
    ($span:expr, $($message:tt)+) => {
        $crate::diagnostics::warning($span, diagnostic_message!($($message)+))
    };
}

pub(crate) fn error(span: impl Into<Span>, message: String) {
    ERRORS.with_borrow_mut(|errors| errors.push(syn::Error::new(span.into(), message)));
}

pub(crate) fn warning(span: impl Into<Span>, message: String) {
    // Out of a macro, in this crate's tests, there's no compiler to take it
    #[cfg(vertigo_macro_warnings)]
    if proc_macro::is_available() {
        proc_macro::Diagnostic::spanned(span.into().unwrap(), proc_macro::Level::Warning, message)
            .emit();
    }
    #[cfg(not(vertigo_macro_warnings))]
    let _ = (span, message);
}

/// Run a macro's expansion. If it reported errors, they replace what it made.
pub(crate) fn expand(
    expansion: impl FnOnce() -> proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Left by an expansion that panicked - rustc goes on to the next macro on the same thread
    take_errors();
    let output = expansion();
    match errors_to_tokens(take_errors()) {
        Some(errors) => errors.into(),
        None => output,
    }
}

fn take_errors() -> Vec<syn::Error> {
    ERRORS.take()
}

fn errors_to_tokens(errors: Vec<syn::Error>) -> Option<TokenStream2> {
    if errors.is_empty() {
        return None;
    }
    Some(errors.iter().map(syn::Error::to_compile_error).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages() -> Vec<String> {
        take_errors().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn message_forms() {
        take_errors();
        let name = "x";
        let owned = String::from("from a value");

        emit_error!(Span::call_site(), "plain");
        emit_error!(Span::call_site(), "formatted {} {:?}", name, 1);
        emit_error!(Span::call_site(), "trailing comma {}", name,);
        emit_error!(Span::call_site(), owned);
        emit_call_site_error!("at call site {}", name);

        assert_eq!(
            messages(),
            [
                "plain",
                "formatted x 1",
                "trailing comma x",
                "from a value",
                "at call site x"
            ]
        );
    }

    #[test]
    fn a_value_is_displayed_not_formatted() {
        take_errors();
        let message = "{not an argument}";

        emit_error!(Span::call_site(), message);

        assert_eq!(messages(), ["{not an argument}"]);
    }

    #[test]
    fn warnings_are_not_errors() {
        take_errors();

        emit_warning!(Span::call_site(), "a warning {}", 1);

        assert!(take_errors().is_empty());
    }

    #[test]
    fn errors_become_compile_errors() {
        assert!(errors_to_tokens(Vec::new()).is_none());

        let errors = vec![
            syn::Error::new(Span::call_site(), "first"),
            syn::Error::new(Span::call_site(), "second"),
        ];
        let tokens = errors_to_tokens(errors).map(|tokens| tokens.to_string());

        assert_eq!(
            tokens.as_deref(),
            Some(
                ":: core :: compile_error ! { \"first\" } \
                 :: core :: compile_error ! { \"second\" }"
            )
        );
    }
}
