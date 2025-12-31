use proc_macro::{Span, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::error::Error;
use std::fs::read_to_string;
use std::io::Write;
use std::{path::PathBuf, process::Command};
use syn::spanned::Spanned;
use syn::{Expr, Lit, parse2};

use crate::get_target_dir::get_target_dir;

/// Add tailwind class to dictionary
pub(crate) fn trace_tailwind(input: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let output = add_to_tailwind(input.into())?;
    Ok(quote! { vertigo::TwClass::new(#output) }.into())
}

/// Bundle tailwind and return injector
pub(crate) fn bundle_tailwind() -> TokenStream2 {
    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        let bundle = if std::env::var("VERTIGO_EXT_TAILWIND").is_ok() {
            // Use external tailwind
            match run_external_tailwind() {
                Ok(output) => output,
                Err(err) => {
                    emit_error!(Span::call_site(), "Tailwind: {}", err);
                    return quote! {};
                }
            }
        } else {
            // Use internal tailwind
            match get_tailwind_classes_file_path() {
                Ok(file_path) => {
                    if let Ok(tailwind_classes) = std::fs::read_to_string(file_path) {
                        let mut tailwind_bundler = tailwind_css::TailwindBuilder::default();
                        for tailwind_classes_row in tailwind_classes.lines() {
                            let _ = tailwind_bundler
                                .trace(tailwind_classes_row.trim_matches('"'), false);
                        }
                        match tailwind_bundler.bundle() {
                            Ok(bundle) => bundle,
                            Err(err) => {
                                emit_error!(Span::call_site(), "Internal tailwind: {}", err);
                                return quote! {};
                            }
                        }
                    } else {
                        return quote! {};
                    }
                }
                Err(err) => {
                    emit_error!(Span::call_site(), "Tailwind: {}", err);
                    return quote! {};
                }
            }
        };
        quote! {
            vertigo::get_driver().register_bundle(#bundle);
        }
    } else {
        quote! {}
    }
}

pub(crate) fn add_to_tailwind(classes: TokenStream2) -> Result<TokenStream2, Box<dyn Error>> {
    let classes_span = classes.span();
    let Ok(input) = parse2::<Expr>(classes) else {
        emit_error!(classes_span, "The macro can only take strings");
        return Ok(quote! {});
    };
    if let Expr::Lit(expr_lit) = &input
        && let Lit::Str(input) = &expr_lit.lit
    {
        let input_str = input.to_token_stream().to_string();
        let input_str = input_str.trim_matches('"');
        let output = if std::env::var("VERTIGO_EXT_TAILWIND").is_ok() {
            // External tailwind doesn't modify class names
            Ok(input_str.to_string())
        } else {
            tailwind_css::TailwindBuilder::default().trace(input_str, false)
        };

        match output {
            Ok(output) => {
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
                return Ok(quote! { #output });
            }
            Err(err) => {
                emit_error!(input.span(), "Tailwind: {}", err.kind.to_string());
            }
        }
    }
    Ok(quote! { #input })
}

fn get_tailwind_classes_dir() -> Result<PathBuf, Box<dyn Error>> {
    let dir = get_target_dir().join("tailwind");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Can't create tailwind subdirectory: {}", err))?;
    Ok(dir)
}

fn get_tailwind_classes_file_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(get_tailwind_classes_dir()?.join("classes.html"))
}

fn run_external_tailwind() -> Result<String, Box<dyn Error>> {
    // Working dir for tailwind
    let tailwind_dir = get_tailwind_classes_dir()?;

    // Create empty source CSS for tailwind
    let input_styles_path = tailwind_dir.join("input.css");
    let mut input_styles_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(input_styles_path)
        .map_err(|err| format!("Unable to create input.css file: {}", err))?;

    // Try to read user-provided input file for tailwind, otherwise, use the default
    let user_tw_input_path = PathBuf::from(Span::call_site().file())
        .parent()
        .ok_or("Unable to get parent directory")?
        .join("tailwind.css");

    if let Ok(user_defined_input) = read_to_string(&user_tw_input_path) {
        // Write custom tailwind input
        write!(input_styles_file, "{user_defined_input}")
            .map_err(|err| format!("Unable to write custom tailwind input to file: {}", err))?;
    } else {
        // Write default tailwind input
        writeln!(input_styles_file, "@import \"tailwindcss\";")
            .map_err(|err| format!("Unable to default tailwind input to file: {}", err))?;
    }

    // Run tailwind and catch stdout with the bundle
    let Ok(command) = Command::new("npm")
        .args([
            "exec",
            "-p",
            "@tailwindcss/cli",
            "--",
            "-i",
            "input.css",
            "--cwd",
            &tailwind_dir.to_string_lossy(),
        ])
        .output()
    else {
        abort_call_site!(
            "Failed to run external tailwind";
            help = "Maybe NPM not installed or version is incompatible?"
        );
    };

    let output = String::from_utf8_lossy(&command.stdout);

    if command.status.success() {
        Ok(output.into_owned())
    } else {
        let err_output = String::from_utf8_lossy(&command.stderr);
        abort_call_site!(
            "Tailwind run failed: {}", err_output;
            help = "To install tailwind run `npm install tailwindcss @tailwindcss/cli`.";
            note = "Tailwind output: {}", output;
        );
    }
}
