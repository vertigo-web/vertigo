use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::cell::Cell;
use std::error::Error;
use std::io::Write;
use syn::spanned::Spanned;
use syn::{Expr, Lit, parse2};

use crate::{trace_tailwind::paths::get_tailwind_classes_file_path, utils::track_build_id};

thread_local! {
    /// Whether the macro being expanded has fed any class to the tailwind bundle
    static CLASSES_TRACED: Cell<bool> = const { Cell::new(false) };
}

/// Runs a macro expansion and, if it fed any class to the tailwind bundle, adds
/// [track_build_id] to its output.
///
/// Also in a library crate, not just in the one with `#[vertigo::main]` - vertigo-cli wipes
/// the classes before each build, so one taking the library from cache would miss them.
/// The flag, not the class itself, as `tw=` classes end up deep inside `dom!`, in places
/// (like a component's struct literal) that can't take an item.
pub(crate) fn with_build_tracking(expand: impl FnOnce() -> TokenStream2) -> TokenStream2 {
    CLASSES_TRACED.set(false);
    let output = expand();

    if CLASSES_TRACED.replace(false) {
        let track_build_id = track_build_id();
        quote! {{
            #track_build_id
            #output
        }}
    } else {
        output
    }
}

/// Add tailwind class to dictionary
pub(crate) fn trace_tailwind(input: TokenStream) -> Result<TokenStream2, Box<dyn Error>> {
    let output = add_to_tailwind(input.into())?;
    Ok(quote! { vertigo::TwClass::new(#output) })
}

pub(crate) fn add_to_tailwind(classes: TokenStream2) -> Result<TokenStream2, Box<dyn Error>> {
    let classes_span = classes.span();
    let Ok(input) = parse2::<Expr>(classes) else {
        emit_error!(classes_span, "The macro can only take strings");
        return Ok(quote! {});
    };
    if let Expr::Lit(expr_lit) = &input
        && let Lit::Str(input_lit) = &expr_lit.lit
    {
        let input_str = input_lit.to_token_stream().to_string();
        let input_str = input_str.trim_matches('"');
        // Tracked also outside of a bundling build, so that the first one after it
        // doesn't take the crate from cache
        CLASSES_TRACED.set(true);
        // Only collect tailwind classes during build
        if std::env::var("VERTIGO_BUNDLE").is_ok() {
            let file_path = get_tailwind_classes_file_path()?;

            // Open the file in append mode
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true) // Create the file if it doesn't exist
                .open(&file_path)?;

            // Write the input string to the file
            writeln!(file, "{input_str}")?;
        }
        // Use output in source code
        return Ok(quote! { #input_lit });
    }
    Ok(quote! { #input })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether an expansion feeding `classes` to the bundle gets `track_build_id`
    fn is_tracked(classes: TokenStream2) -> Result<bool, Box<dyn Error>> {
        let mut added = Ok(quote! {});
        let output = with_build_tracking(|| {
            added = add_to_tailwind(classes);
            quote! {}
        });
        added?;
        Ok(output.to_string().contains("VERTIGO_BUILD_ID"))
    }

    #[test]
    fn test_literal_class_is_tracked() -> Result<(), Box<dyn Error>> {
        assert!(is_tracked(quote! { "flex" })?);
        Ok(())
    }

    #[test]
    fn test_expression_is_not_tracked() -> Result<(), Box<dyn Error>> {
        assert!(!is_tracked(quote! { my_class })?);
        Ok(())
    }

    #[test]
    fn test_tracking_does_not_leak_to_next_expansion() -> Result<(), Box<dyn Error>> {
        with_build_tracking(|| {
            CLASSES_TRACED.set(true);
            quote! {}
        });

        assert!(!is_tracked(quote! { my_class })?);
        Ok(())
    }
}
