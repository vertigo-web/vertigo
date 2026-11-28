use proc_macro::{Span, TokenStream};
use quote::quote;
use std::{error::Error, path::PathBuf};

use crate::{get_target_dir::get_target_wasm_dir_with, utils::track_build_id};

use super::wasm_path::WasmPath;

pub(crate) fn include_static_inner(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let file_path = Span::call_site().file().into();

    match bundle_file(file_path, path) {
        Ok(hash) => {
            // Also in a library crate, not just in the one with `#[vertigo::main]` - otherwise
            // a build taking the library from cache would miss its files
            let track_build_id = track_build_id();
            quote! {{
                #track_build_id
                vertigo::get_driver().public_build_path(#hash)
            }}
            .into()
        }
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            quote! { "".to_string() }.into()
        }
    }
}

fn bundle_file(mut file_path: PathBuf, file: String) -> Result<String, Box<dyn Error>> {
    // rust-analyzer gives a macro no source file, so the file can't be found next to it. It
    // doesn't bundle anything either, so there is nothing to check.
    if file_path.as_os_str().is_empty() && std::env::var("VERTIGO_BUNDLE").is_err() {
        return Ok(String::default());
    }

    // Remove quoting from file name
    let file_inner = &file[1..file.len() - 1];

    // Check source path
    file_path.pop();
    file_path.push(file_inner);

    let file_path = WasmPath::new(file_path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.as_string()).into());
    }

    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        // Intermediate directory
        let file_name = file_path.file_name()?;
        let file_static_target = get_target_wasm_dir_with(&["static", "included", &file_name]);

        let file_path_content = file_path.read()?;
        let hash = file_static_target.save_with_hash(file_path_content.as_slice())?;

        // Final public path in the build (can be later mangled by dynamic path dispatch though)
        let Ok(public_path) = std::env::var("VERTIGO_PUBLIC_PATH") else {
            return Err("The environment variable \"VERTIGO_PUBLIC_PATH\" is missing".into());
        };

        let http_path = format!("{public_path}/{hash}");

        Ok(http_path)
    } else {
        // Don't produce assets if not in build mode
        Ok(String::default())
    }
}

#[cfg(test)]
mod tests {
    use super::bundle_file;
    use std::{error::Error, path::PathBuf};

    fn this_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/include_static.rs")
    }

    #[test]
    fn file_next_to_source() -> Result<(), Box<dyn Error>> {
        assert_eq!(bundle_file(this_file(), "\"./lib.rs\"".to_string())?, "");
        Ok(())
    }

    #[test]
    fn missing_file() {
        let result = bundle_file(this_file(), "\"./missing.webp\"".to_string());
        assert!(
            matches!(&result, Err(err) if err.to_string().starts_with("File does not exist: ")),
            "{result:?}"
        );
    }

    #[test]
    fn no_source_file_in_rust_analyzer() -> Result<(), Box<dyn Error>> {
        // `Span::file()` is empty in rust-analyzer
        assert_eq!(
            bundle_file(PathBuf::new(), "\"./missing.webp\"".to_string())?,
            ""
        );
        Ok(())
    }
}
