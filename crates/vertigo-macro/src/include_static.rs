use std::path::PathBuf;

use super::wasm_path::WasmPath;

pub fn include_static(mut file_path: PathBuf, file: String) -> Result<String, String> {
    let file_inner = &file[1 .. file.len() - 1];

    file_path.pop();
    file_path.push(file_inner);

    let file_path = WasmPath::new(file_path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.as_string()));
    }

    if cfg!(debug_assertions) {
        Ok(String::default())
    } else {
        let file_name = file_path.file_name();
        let file_static_target = {
            let mut target_path = WasmPath::new(
                PathBuf::from(
                    std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
                        format!(
                            "target/{}/release",
                            std::env::var("TARGET").unwrap_or_else(|_| "wasm32-unknown-unknown".to_string())
                        )
                    })
                )
            );
            target_path.push(&["static", "included", file_name.as_str()]);
            target_path
        };

        let file_path_content = file_path.read();
        let hash = file_static_target.save_with_hash(file_path_content.as_slice());
        Ok(hash)
    }
}
