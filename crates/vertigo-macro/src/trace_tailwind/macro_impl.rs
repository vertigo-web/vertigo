use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::error::Error;
use std::io::Write;
use syn::spanned::Spanned;
use syn::{Expr, Lit, parse2};

use crate::trace_tailwind::paths::get_tailwind_classes_file_path;

/// Add tailwind class to dictionary
pub(crate) fn trace_tailwind(input: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let output = add_to_tailwind(input.into())?;
    Ok(quote! { vertigo::TwClass::new(#output) }.into())
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
        let input_str = input_lit.value();
        collect_tailwind_classes(&input_str, false)?;
        // Use output in source code
        return Ok(quote! { #input_lit });
    }
    Ok(quote! { #input })
}

pub(crate) fn collect_tailwind_classes(input_str: &str, is_raw_css: bool) -> Result<(), Box<dyn Error>> {
    // Only collect tailwind classes during build
    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        let file_path = get_tailwind_classes_file_path()?;

        // Open the file in append mode
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true) // Create the file if it doesn't exist
            .open(&file_path)?;

        if is_raw_css {
            // For raw CSS, we only care about variables
            // Extract var usage and write it in a way that validate.rs can identify as non-class
            let re = regex::Regex::new(r"var\((--[a-zA-Z0-9_-]+)\)").unwrap();
            for cap in re.captures_iter(input_str) {
                writeln!(file, "var({})", &cap[1])?;
            }
        } else {
            // Write the input string to the file
            writeln!(file, "{input_str}")?;
        }
    }
    Ok(())
}
