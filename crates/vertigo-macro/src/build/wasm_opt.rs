use crate::logs::{log_ok, log_error};
use super::config::WasmPath;


pub fn run_wasm_opt(from: &WasmPath, to: &WasmPath) -> bool {

    //wasm-opt -Os --strip-debug -o ./demo/build/vertigo_demo.wasm ${CARGO_MAKE_CRATE_TARGET_DIRECTORY}/wasm32-unknown-unknown/release/vertigo_demo.wasm

    let wasm_opt_result = std::process::Command::new("wasm-opt")
        .args([
            "-Os",
            "--strip-debug",
            "-o",
            to.string().as_str(),
            from.string().as_str()
        ])
        .output();

    println!(
        r#"I am trying to run "wasm-opt -Os --strip-debug -o {} {}""#,
        to.string(),
        from.string()
    );

    match wasm_opt_result {
        Ok(out) => {
            log_ok("wasm-opt ok!!!");

            let out = out.stdout.to_vec();
            let out = String::from_utf8(out);

            match out {
                Ok(out) => {
                    log_ok(format!("output: {out}"));
                },
                Err(_) => {
                    log_error("output error: invalid sequence UTF-8");
                }
            }
            true
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
