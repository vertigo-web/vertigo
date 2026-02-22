use proc_macro::Span;
use proc_macro_error::{abort_call_site, emit_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::error::Error;
use std::fs::read_to_string;
use std::io::Write;
use std::{path::PathBuf, process::Command};

use crate::trace_tailwind::paths::{get_tailwind_binary_path, get_tailwind_classes_dir};
use crate::trace_tailwind::validate::validate_tailwind_classes;

/// Bundle tailwind and return injector
pub(crate) fn bundle_tailwind() -> TokenStream2 {
    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        let bundle = match run_external_tailwind() {
            Ok(output) => output,
            Err(err) => {
                emit_error!(Span::call_site(), "Tailwind: {}", err);
                return quote! {};
            }
        };

        if let Err(err) = validate_tailwind_classes(&bundle) {
            emit_error!(
                proc_macro::Span::call_site(),
                "Failed to validate Tailwind classes: {}",
                err
            );
        }

        quote! {
            vertigo::get_driver().register_bundle(#bundle);
        }
    } else {
        quote! {}
    }
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

    let tailwind_executable = get_tailwind_binary_path()?;

    let tailwind_executable_abs = tailwind_executable
        .canonicalize()
        .unwrap_or(tailwind_executable);

    // Run tailwind and catch stdout with the bundle
    let command = match Command::new(&tailwind_executable_abs)
        .args(["-i", "input.css"])
        .current_dir(&tailwind_dir)
        .output()
    {
        Ok(cmd) => cmd,
        Err(err) => {
            abort_call_site!(
                "Failed to run external tailwind: {}", err;
                help = "Path was: {}", tailwind_executable_abs.display()
            );
        }
    };

    let output = String::from_utf8_lossy(&command.stdout);

    if command.status.success() {
        Ok(output.into_owned())
    } else {
        let err_output = String::from_utf8_lossy(&command.stderr);
        abort_call_site!(
            "Tailwind run failed: {}", err_output;
            note = "Tailwind output: {}", output;
        );
    }
}
