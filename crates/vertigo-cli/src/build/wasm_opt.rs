use std::{fs, process::Command};

use super::wasm_path::WasmPath;

pub fn run_wasm_opt(from: &WasmPath, to: &WasmPath) -> bool {
    let from_str = from.as_string();
    let to_str = to.as_string();

    let mut wasm_opt_command = Command::new("wasm-opt");
    wasm_opt_command.args(["-Os", "--strip-debug", "-o", &to_str, &from_str]);

    log::info!("Running: {:?}", wasm_opt_command);

    let wasm_opt_status = wasm_opt_command.status();

    match wasm_opt_status {
        Ok(status) if status.success() => {
            let in_size = size(&from_str);
            let out_size = size(&to_str);
            let percent = 100 * out_size / in_size;
            log::info!("WASM optimized: {in_size}K -> {out_size}K ({percent}%)");
            true
        }
        Ok(_) => {
            log::error!("WASM optimization failed");
            false
        }
        Err(error) => {
            log::error!(
                r#"

                WARNING: Can't perform wasm-opt: {error}
                Your WASM package will be left unoptimized.

                HINT: If you don't have "wasm-opt" command in your system,
                install Binaryen package: https://github.com/WebAssembly/binaryen

            "#
            );

            false
        }
    }
}

fn size(path: &str) -> u64 {
    fs::metadata(path)
        .map(|md| md.len() / 1_024)
        .unwrap_or_default()
}
