use proc_macro::{Span, TokenStream};
use quote::quote;
use std::path::PathBuf;

use crate::get_target_dir::get_target_wasm_dir_with;

use super::wasm_path::WasmPath;

pub(crate) fn include_static_inner(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let file_path = Span::call_site().file().into();

    match bundle_file(file_path, path) {
        Ok(hash) => quote! { vertigo::get_driver().public_build_path(#hash) }.into(),
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            quote! { "".to_string() }.into()
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

    if std::env::var("VERTIGO_BUNDLE").is_ok() {
        // Intermediate directory
        let file_name = file_path.file_name();
        let file_static_target = get_target_wasm_dir_with(&["static", "included", &file_name]);

        let file_path_content = file_path.read();
        let hash = file_static_target.save_with_hash(file_path_content.as_slice());

        // Final public path in the build (can be later mangled by dynamic path dispatch though)
        let Ok(public_path) = std::env::var("VERTIGO_PUBLIC_PATH") else {
            return Err(r#"The environment variable "VERTIGO_PUBLIC_PATH" is missing"#.to_string());
        };

        let http_path = format!("{public_path}/{hash}");

        Ok(http_path)
    } else {
        // Don't produce assets if not in build mode
        Ok(String::default())
    }
}
