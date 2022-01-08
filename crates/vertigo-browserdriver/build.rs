use std::fs;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wasm_run.js");

    let glue = fs::read_to_string("src/wasm_run.js")
        .expect("src/wasm_run.js");

    let filename = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("../../../wasm_run.js");

    fs::write(filename, glue)
        .expect("Couldn't write wasm_run.js!");
}
