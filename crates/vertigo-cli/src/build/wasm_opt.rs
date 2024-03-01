use super::wasm_path::WasmPath;

pub fn run_wasm_opt(from: &WasmPath, to: &WasmPath) -> bool {
    log::info!(
        r#"Running "wasm-opt -Os --strip-debug -o {} {}""#,
        to.as_string(),
        from.as_string()
    );

    let wasm_opt_status = std::process::Command::new("wasm-opt")
        .args([
            "-Os",
            "--strip-debug",
            "-o",
            to.as_string().as_str(),
            from.as_string().as_str(),
        ])
        .status();

    match wasm_opt_status {
        Ok(status) if status.success() => {
            log::info!("WASM optimized");
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
