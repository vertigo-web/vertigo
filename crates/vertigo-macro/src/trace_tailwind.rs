use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse2, Expr, Lit};

use crate::get_target_dir::get_target_dir;

/// Add tailwind class to dictionary
pub(crate) fn trace_tailwind(input: TokenStream) -> TokenStream {
    let output = add_to_tailwind(input.into());
    quote! { vertigo::TwClass::new(#output) }.into()
}

/// Bundle tailwind and return injector
pub(crate) fn bundle_tailwind() -> TokenStream2 {
    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        let file_path = get_tailwind_classes_file_path();

        if let Ok(tailwind_classes) = std::fs::read_to_string(file_path) {
            let mut tailwind = tailwind_css::TailwindBuilder::default();
            for tailwind_classes_row in tailwind_classes.lines() {
                let _ = tailwind.trace(tailwind_classes_row.trim_matches('"'), false);
            }
            let bundle = tailwind.bundle().unwrap();
            quote! {
                vertigo::get_driver().register_bundle(#bundle);
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    }
}

pub(crate) fn add_to_tailwind(classes: TokenStream2) -> TokenStream2 {
    let input: Expr = parse2(classes).unwrap();
    if let Expr::Lit(expr_lit) = &input {
        if let Lit::Str(input) = &expr_lit.lit {
            let input_str = input.to_token_stream().to_string();
            let input_str = input_str.trim_matches('"');
            let output = tailwind_css::TailwindBuilder::default().trace(input_str, false);

            match output {
                Ok(output) => {
                    // Only collect tailwind classes during build
                    if std::env::var("VERTIGO_BUNDLE").is_ok() {
                        let file_path = get_tailwind_classes_file_path();

                        // Open the file in append mode
                        let mut file = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true) // Create the file if it doesn't exist
                            .open(&file_path)
                            .expect("Unable to open file");

                        use std::io::Write;
                        // Write the input string to the file
                        writeln!(file, "{input_str}").expect("Unable to write to file");
                    }
                    // Use output in source code
                    return quote! { #output };
                }
                Err(err) => {
                    emit_error!(input.span(), "Tailwind: {}", err.kind.to_string());
                }
            }
        }
    }
    quote! { #input }
}

fn get_tailwind_classes_file_path() -> PathBuf {
    get_target_dir().join("tailwind_classes.txt")
}
