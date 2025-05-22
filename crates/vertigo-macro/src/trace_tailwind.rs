use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse2, Expr, Lit};

pub(crate) fn trace_tailwind(input: TokenStream) -> TokenStream {
    let output = add_to_tailwind(input.into());
    quote! { vertigo::TwClass::new(#output) }.into()
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
                    let out_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
                        format!(
                            "target/{}/release",
                            std::env::var("TARGET")
                                .unwrap_or_else(|_| "wasm32-unknown-unknown".to_string())
                        )
                    });
                    let file_path = std::path::PathBuf::from(out_dir).join("tailwind_classes.txt");

                    // Open the file in append mode
                    let mut file = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true) // Create the file if it doesn't exist
                        .open(&file_path)
                        .expect("Unable to open file");

                    use std::io::Write;
                    // Write the input string to the file
                    writeln!(file, "{input_str}").expect("Unable to write to file");
                    // Use output in source code
                    return quote! { #output }
                },
                Err(err) => {
                    emit_error!(input.span(), err.kind.to_string());
                }
            }
        }
    }
    quote! { #input }
}
