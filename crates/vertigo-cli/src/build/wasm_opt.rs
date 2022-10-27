use crate::logs::{log_ok, log_error};
use super::wasm_path::WasmPath;

pub fn run_wasm_opt(from: &WasmPath, to: &WasmPath) -> bool {
    log_ok(format!(
        r#"Running "wasm-opt -Os --strip-debug -o {} {}""#,
        to.as_string(),
        from.as_string()
    ));

    let wasm_opt_status = std::process::Command::new("wasm-opt")
        .args([
            "-Os",
            "--strip-debug",
            "-o",
            to.as_string().as_str(),
            from.as_string().as_str()
        ])
        .status();

    match wasm_opt_status {
        Ok(status) if status.success() => {
            log_ok("WASM optimized");
            true
        },
        Ok(_) => {
            log_error("WASM optimization failed");
            false
        },
        Err(error) => {
            let message = format!(r#"


                Error running wasm-opt: {error}
                Your WASM package will be left unoptimized.

                HINT: If you don't have "wasm-opt" command in your system,
                install Binaryen package: https://github.com/WebAssembly/binaryen


            "#);
            log_error(message);

            false
        }
    }
}
