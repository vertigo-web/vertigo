use proc_macro::{Span, TokenStream};
use quote::quote;
use std::path::PathBuf;

use super::wasm_path::WasmPath;

pub(crate) fn include_static_inner(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let file_path = Span::call_site().source_file().path();

    match bundle_file(file_path, path) {
        Ok(hash) => quote! { #hash }.into(),
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            let empty = "";
            quote! { #empty }.into()
        }
    }
}

fn bundle_file(mut file_path: PathBuf, file: String) -> Result<String, String> {
    // Remove quoting from file name
    let file_inner = &file[1..file.len() - 1];

    // Check source path
    file_path.pop();
    file_path.push(file_inner);

    let file_path = WasmPath::new(file_path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.as_string()));
    }

    if cfg!(debug_assertions) {
        // Don't produce assets if not in build mode
        Ok(String::default())
    } else {
        // Intermediate directory
        let file_name = file_path.file_name();
        let file_static_target = {
            let mut target_path = WasmPath::new(PathBuf::from(
                std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
                    format!(
                        "target/{}/release",
                        std::env::var("TARGET")
                            .unwrap_or_else(|_| "wasm32-unknown-unknown".to_string())
                    )
                }),
            ));
            target_path.push(&["static", "included", file_name.as_str()]);
            target_path
        };

        let file_path_content = file_path.read();
        let hash = file_static_target.save_with_hash(file_path_content.as_slice());

        // Final public path
        let Ok(public_path) = std::env::var("VERTIGO_PUBLIC_PATH") else {
            return Err(r#"The environment variable "VERTIGO_PUBLIC_PATH" is missing"#.to_string());
        };

        let http_path = format!("{public_path}/{hash}");

        Ok(http_path)
    }
}
