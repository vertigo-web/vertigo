use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
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
        let input_str = input_lit.to_token_stream().to_string();
        let input_str = input_str.trim_matches('"');
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
