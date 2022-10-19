use lazy_static::lazy_static;
use std::{sync::Mutex, path::PathBuf};

mod config;
mod wasm_opt;
mod find_target;
mod get_html;
mod hash;

pub use config::{EnvConfigIn, WasmPath};

lazy_static! {
    static ref IS_STARTED: Mutex<Option<EnvConfigIn>> = Mutex::new(None);
}

pub fn get_config() -> EnvConfigIn {
    let mut config_guard = IS_STARTED.lock().unwrap();

    if let Some(config) = &*config_guard {
        return config.clone();
    }

    let config = match envy::from_env::<EnvConfigIn>() {
        Ok(config) => config,
        Err(error) => {
            panic!("Incorrect environment variables error={error}");
        }
    };

    *config_guard = Some(config.clone());

    if cfg!(debug_assertions) {
        println!("Debugging enabled");
        prepare_build(&config);
    }

    drop(config_guard);
    config
}

fn prepare_build(config: &EnvConfigIn) {
    let static_path = config.get_static_path();
    static_path.remove_dir_all();
    static_path.create_dir_all();


    let wasm_run_hash_name = config
        .new_path_in_static_make(&["wasm_run.js"])
        .save_with_hash(include_str!("../wasm_run.js").as_bytes());


    let wasm_path_target = find_target::find_wasm_from_target();    
    let wasm_path = config.new_path_in_static_from(&wasm_path_target);

    let wasm_path_hash = if wasm_opt::run_wasm_opt(&wasm_path_target, &wasm_path) {
        //success
        let wasm_path_hash = wasm_path.save_with_hash(wasm_path.read().as_slice());
        wasm_path.remove_file();
        wasm_path_hash
    } else {
        //copy without optimisation
        let wasm_content = wasm_path_target.read();
        wasm_path.save_with_hash(wasm_content.as_slice())
    };

    let html_content = get_html::get_html(
        config,
        wasm_run_hash_name,
        wasm_path_hash
    );

    config.new_path_in_static_make(&["index.html"]).save(html_content.as_bytes());

}

pub fn build_static() {
    let static_path = std::env::var("static_path");
    
    if let Ok(static_path) = static_path {
        if !static_path.is_empty() {
            let _ = get_config();
        }
    }
}

pub fn include_static(mut file_path: PathBuf, file: String) -> Result<String, String> {
    let file_inner = &file[1 .. file.len() - 1];

    file_path.pop();
    file_path.push(file_inner);

    let file_path = WasmPath::new(file_path);

    if !file_path.is_exist() {
        return Err(format!("File does not exist, file={}", file_path.string()));
    }

    let static_path = std::env::var("static_path");
    
    if static_path.is_ok() {
        let file_static_target = get_config().new_path_in_static_from(&file_path);

        let file_path_content = file_path.read();
        let hash = file_static_target.save_with_hash(file_path_content.as_slice());
        Ok(hash)
    } else {
        Ok(String::default())
    }
}
