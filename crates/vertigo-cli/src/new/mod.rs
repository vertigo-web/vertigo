use clap::Args;
use include_dir::{include_dir, Dir};
use std::{path::Path, fs};

use crate::logs::{log_ok, log_error};

static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/new/template");

#[derive(Args)]
pub struct NewOpts {
    pub package_name: String,
    #[arg(long, default_value_t = {"./".to_string()})]
    pub dest_dir: String,
}

pub fn run(opts: NewOpts) -> Result<(), i32> {
    log_ok(format!("Creating {}", opts.package_name));

    let target_path = Path::new(&opts.dest_dir).join(&opts.package_name);

    // Check if dir is empty or non-existent
    if let Ok(mut dir) = target_path.read_dir() {
        if dir.next().is_some() {
            log_error(format!("Destination dir ({}) is not empty", target_path.to_string_lossy()));
            return Err(-1)
        }
    }

    // Create directory
    if let Err(err) = fs::create_dir_all(&target_path) {
        log_error(format!("Can't create directory {}: {}", target_path.to_string_lossy(), err));
        return Err(-2)
    };

    // Paste files into it
    if let Err(err) = TEMPLATE.extract(Path::new(&opts.dest_dir).join(&opts.package_name)) {
        log_error(format!("Can't unpack vertigo stub to {}: {}", target_path.to_string_lossy(), err));
        return Err(-3)
    };

    // Replace package name in original Cargo.toml
    let cargo_toml = TEMPLATE.get_file("Cargo.toml").unwrap();
    let cargo_toml_content = cargo_toml.contents_utf8().unwrap();

    if let Err(err) = fs::write(target_path.join("Cargo.toml"), cargo_toml_content.replace("my_app", &opts.package_name)) {
        log_error(format!("Can't write to Cargo.toml: {}", err));
        return Err(-4)
    };

    Ok(())
}
