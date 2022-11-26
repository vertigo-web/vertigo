pub mod build_opts;
mod cargo_build;
mod wasm_opt;
mod find_target;
//TODO: To be eventually moved to separate 'commons' lib
mod wasm_path;
mod cargo_workspace;

use cargo_workspace::infer_package_name;
use std::path::PathBuf;
use wasm_path::WasmPath;

pub use build_opts::BuildOpts;

pub fn run(opts: BuildOpts) -> Result<(), i32> {
    let package_name = match opts.package_name.as_deref() {
        Some(name) => name.to_string(),
        None => match infer_package_name() {
            Ok(name) => {
                log::info!("Inferred package name = {}", name);
                name
            },
            Err(err) => {
                log::error!("{}", err);
                return Err(-1)
            },
        },
    };

    let dest_dir = WasmPath::new(PathBuf::from(&opts.dest_dir));

    // Clean destination

    dest_dir.remove_dir_all();
    dest_dir.create_dir_all();

    // Delete rlibs to re-generate static files

    find_target::find_package_rlib_in_target(&package_name).remove_file();

    // Run build

    let target_path = match cargo_build::run_cargo_build(&package_name, &opts.public_path) {
        Ok(path) => path,
        Err(_) => return Err(-2),
    };

    // Get wasm_run.js and index.template.html from vertigo build

    let vertigo_statics_dir = target_path.join("static");

    let run_script_content = std::fs::read(vertigo_statics_dir.join("wasm_run.js"))
        .expect("No wasm_run in statics directory");

    let run_script_hash_name = opts
        .new_path_in_static_make(&["wasm_run.js"])
        .save_with_hash(&run_script_content);

    // Copy .wasm to destination

    let wasm_path_target = find_target::find_wasm_in_target(&package_name);
    let wasm_path = opts.new_path_in_static_from(&wasm_path_target);

    // Optimize .wasm

    let wasm_path_hash = if wasm_opt::run_wasm_opt(&wasm_path_target, &wasm_path) {
        // optimized
        let wasm_path_hash = wasm_path.save_with_hash(wasm_path.read().as_slice());
        wasm_path.remove_file();
        wasm_path_hash
    } else {
        // copy without optimization
        let wasm_content = wasm_path_target.read();
        wasm_path.save_with_hash(wasm_content.as_slice())
    };

    // Generate index.html in destination

    let index_template = std::fs::read_to_string(vertigo_statics_dir.join("index.template.html"))
        .expect("No index.template.html in statics directory");

    let html_content = index_template
        .replace("{wasm_run_path}", &opts.public_path_to(run_script_hash_name))
        .replace("{wasm_lib_path}", &opts.public_path_to(wasm_path_hash));

    opts.new_path_in_static_make(&["index.html"]).save(html_content.as_bytes());

    // Copy statics generated by dom macro invocations

    if let Ok(dir) = std::fs::read_dir(vertigo_statics_dir.join("included")) {
        dir.for_each(|entry| {
            if let Ok(entry) = entry {
                let src_file_path = WasmPath::new(entry.path());
                let content = src_file_path.read();
                let dest_file_path = opts.new_path_in_static_from(&src_file_path);
                dest_file_path.save(&content);
            }
        });
    }

    Ok(())
}
