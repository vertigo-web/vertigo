use std::{fs, process::Command, sync::OnceLock};

use super::wasm_path::WasmPath;
use crate::serve::brotli_compress;

/// Features that rustc enables by default for the `wasm32-unknown-unknown` target.
///
/// Normally `wasm-opt` picks these up from the `target_features` custom section emitted
/// by LLVM, but that section is dropped whenever the binary is stripped
/// (`strip = true` in the cargo profile, which is a very common setting for wasm builds).
/// Without it `wasm-opt` falls back to its own default feature set and refuses to validate
/// the module, e.g.:
///
/// ```text
/// [wasm-validator error in function 1765] unexpected false: memory.copy operations
/// require bulk memory operations [--enable-bulk-memory-opt]
/// ```
///
/// So we always pass them explicitly.
const WASM_FEATURES: &[&str] = &[
    "--enable-bulk-memory",
    "--enable-bulk-memory-opt",
    "--enable-nontrapping-float-to-int",
    "--enable-sign-ext",
    "--enable-mutable-globals",
    "--enable-reference-types",
    "--enable-multivalue",
];

/// Optimization level passed to `wasm-opt`.
///
/// `-Os` rather than a speed level, and measured rather than assumed. Running the whole
/// `tests/dom-bench` suite three times at each level, same source tree, `-O4` came out 0.5%
/// slower at the median with every single workload's ratio inside its own run-to-run spread
/// and 0.25% larger (262000 bytes against 261351). There is little left for `wasm-opt` to
/// do once rustc has run LTO over the crate graph, so size is the only axis where the choice
/// still shows up, and `-Os` is the one that wins it.
const OPT_LEVEL: &str = "-Os";

pub fn run_wasm_opt(from: &WasmPath, to: &WasmPath) -> bool {
    let from_str = from.as_string();
    let to_str = to.as_string();

    let mut wasm_opt_command = Command::new("wasm-opt");
    wasm_opt_command.args(supported_features());
    wasm_opt_command.args([OPT_LEVEL, "--strip-debug", "-o", &to_str, &from_str]);

    log::info!("Running: {wasm_opt_command:?}");

    let wasm_opt_status = wasm_opt_command.status();

    match wasm_opt_status {
        Ok(status) if status.success() => {
            let in_size = size(&from_str);
            let out_size = size(&to_str);
            let percent = format_percent(out_size, in_size);
            log::info!("WASM optimized: {in_size}K -> {out_size}K ({percent}%)");
            log_precompressed_size(&to_str, in_size);
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

/// Subset of [`WASM_FEATURES`] understood by the installed `wasm-opt`.
///
/// Older Binaryen releases reject unknown options, so flags missing from `--help`
/// are filtered out instead of failing the whole optimization step.
fn supported_features() -> &'static Vec<&'static str> {
    static FEATURES: OnceLock<Vec<&'static str>> = OnceLock::new();

    FEATURES.get_or_init(|| {
        let help = Command::new("wasm-opt").arg("--help").output();

        let help = match help {
            Ok(output) => {
                let mut help = String::from_utf8_lossy(&output.stdout).into_owned();
                help.push_str(&String::from_utf8_lossy(&output.stderr));
                help
            }
            // wasm-opt is missing or unusable - run_wasm_opt will report it
            Err(_) => return Vec::new(),
        };

        WASM_FEATURES
            .iter()
            .copied()
            .filter(|feature| help.contains(feature))
            .collect()
    })
}

/// What `serve --precompress` would send, to show it's worth it
fn log_precompressed_size(path: &str, in_size: u64) {
    let Ok(content) = fs::read(path) else {
        return;
    };

    match brotli_compress(&content) {
        Ok(brotli) => {
            let out_size = content.len() as u64 / 1_024;
            let brotli_size = brotli.len() as u64 / 1_024;
            let percent = format_percent(brotli_size, out_size);
            let total_percent = format_percent(brotli_size, in_size);
            log::info!(
                "When pre-compressed: {brotli_size}K ({percent}%, {total_percent}% in total) - serve with --precompress"
            );
        }
        Err(error) => log::warn!("Can't compress the WASM with brotli: {error}"),
    }
}

/// With a decimal place below 10%, where a whole number would round a big shrink to 0%
fn format_percent(part: u64, whole: u64) -> String {
    let percent = 100.0 * part as f64 / whole.max(1) as f64;

    if percent < 9.95 {
        format!("{percent:.1}")
    } else {
        format!("{percent:.0}")
    }
}

fn size(path: &str) -> u64 {
    fs::metadata(path)
        .map(|md| md.len() / 1_024)
        .unwrap_or_default()
}
