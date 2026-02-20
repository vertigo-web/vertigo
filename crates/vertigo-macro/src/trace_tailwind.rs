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
        let bundle = match run_external_tailwind() {
            Ok(output) => output,
            Err(err) => {
                emit_error!(Span::call_site(), "Tailwind: {}", err);
                return quote! {};
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

fn get_tailwind_classes_dir() -> Result<PathBuf, Box<dyn Error>> {
    let dir = get_target_dir().join("tailwind");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Can't create tailwind subdirectory: {}", err))?;
    Ok(dir)
}

fn get_tailwind_classes_file_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(get_tailwind_classes_dir()?.join("classes.html"))
}

fn get_tailwind_binary_path() -> Result<PathBuf, Box<dyn Error>> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let binary_name = match (os, arch) {
        ("windows", "x86_64") => "tailwindcss-windows-x64.exe",
        ("windows", "aarch64") => "tailwindcss-windows-arm64.exe",
        ("macos", "x86_64") => "tailwindcss-macos-x64",
        ("macos", "aarch64") => "tailwindcss-macos-arm64",
        ("linux", "x86_64") => "tailwindcss-linux-x64",
        ("linux", "aarch64") => "tailwindcss-linux-arm64",
        _ => {
            return Err(format!(
                "Unsupported OS or architecture for standalone Tailwind CLI: {os}-{arch}"
            )
            .into());
        }
    };

    let tailwind_version =
        std::env::var("VERTIGO_TAILWIND_VERSION").unwrap_or_else(|_| "v4.2.0".to_string());

    let cache_dir = get_target_dir()
        .join("tailwind_cli_cache")
        .join(&tailwind_version);
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    let filename = if os == "windows" {
        "tailwindcss.exe"
    } else {
        "tailwindcss"
    };
    let executable_path = cache_dir.join(filename);

    if !executable_path.exists() {
        let url = format!(
            "https://github.com/tailwindlabs/tailwindcss/releases/download/{tailwind_version}/{binary_name}"
        );

        println!(
            "Downloading Tailwind CSS CLI from {} to {}",
            url,
            executable_path.display()
        );

        let mut response = reqwest::blocking::get(&url)?;
        if !response.status().is_success() {
            return Err(format!(
                "Failed to download Tailwind CLI: HTTP {}",
                response.status()
            )
            .into());
        }

        let mut file = std::fs::File::create(&executable_path)?;
        std::io::copy(&mut response, &mut file)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&executable_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&executable_path, perms)?;
        }
    }

    Ok(executable_path)
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
