use clap::Args;
use include_dir::{include_dir, Dir};
use std::{fs, path::Path};

use crate::commons::ErrorCode;

static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/new/template");

#[derive(Args)]
pub struct NewOpts {
    pub package_name: String,
    #[arg(long, default_value_t = {"./".to_string()})]
    pub dest_dir: String,
}

pub fn run(opts: NewOpts) -> Result<(), ErrorCode> {
    log::info!("Creating {}", opts.package_name);

    let target_path = Path::new(&opts.dest_dir).join(&opts.package_name);

    // Check if dir is empty or non-existent
    if let Ok(mut dir) = target_path.read_dir() {
        if dir.next().is_some() {
            log::error!(
                "Destination dir ({}) is not empty",
                target_path.to_string_lossy()
            );
            return Err(ErrorCode::NewProjectDirNotEmpty);
        }
    }

    // Create directory
    if let Err(err) = fs::create_dir_all(&target_path) {
        log::error!(
            "Can't create directory {}: {}",
            target_path.to_string_lossy(),
            err
        );
        return Err(ErrorCode::NewProjectCantCreateDir);
    };

    // Paste files into it
    if let Err(err) = TEMPLATE.extract(Path::new(&opts.dest_dir).join(&opts.package_name)) {
        log::error!(
            "Can't unpack vertigo stub to {}: {}",
            target_path.to_string_lossy(),
            err
        );
        return Err(ErrorCode::NewProjectCantUnpackStub);
    };

    // Remove Cargo.toml_
    // (cargo packaging does not permit adding second Cargo.toml file)
    if let Err(err) = fs::remove_file(target_path.join("Cargo.toml_")) {
        log::error!("Can't rename to Cargo.toml_ to Cargo.toml: {err}");
        return Err(ErrorCode::NewProjectCanCreateCargoToml);
    };

    // Save Cargo.toml with replaced package name
    let cargo_toml = TEMPLATE.get_file("Cargo.toml_").unwrap();
    let cargo_toml_content = cargo_toml.contents_utf8().unwrap();

    if let Err(err) = fs::write(
        target_path.join("Cargo.toml"),
        cargo_toml_content.replace("my_app", &opts.package_name),
    ) {
        log::error!("Can't write to Cargo.toml: {err}");
        return Err(ErrorCode::NewProjectCanWriteToCargoToml);
    };

    Ok(())
}
