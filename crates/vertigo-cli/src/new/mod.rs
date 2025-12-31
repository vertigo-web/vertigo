use clap::Args;
use derive_more::Display;
use include_dir::{Dir, include_dir};
use std::{fs, path::Path};
use walkdir::WalkDir;

use crate::commons::ErrorCode;

#[derive(clap::ValueEnum, Clone, Default, Display)]
pub enum Template {
    #[display("fullstack")]
    Fullstack,
    #[display("frontend")]
    #[default]
    Frontend,
}

impl Template {
    pub fn get_dir(&self) -> Dir<'_> {
        match self {
            Template::Fullstack => include_dir!("$CARGO_MANIFEST_DIR/src/new/fs_template"),
            Template::Frontend => include_dir!("$CARGO_MANIFEST_DIR/src/new/fe_template"),
        }
    }
}

#[derive(Args)]
pub struct NewOpts {
    pub project_name: String,
    #[arg(short, long, default_value_t = {Template::default()})]
    pub template: Template,
    #[arg(long, default_value_t = {"./".to_string()})]
    pub dest_dir: String,
}

pub fn run(opts: NewOpts) -> Result<(), ErrorCode> {
    log::info!("Creating {}", opts.project_name);

    let target_path = Path::new(&opts.dest_dir).join(&opts.project_name);

    // Check if dir is empty or non-existent
    if let Ok(mut dir) = target_path.read_dir()
        && dir.next().is_some()
    {
        log::error!(
            "Destination dir ({}) is not empty",
            target_path.to_string_lossy()
        );
        return Err(ErrorCode::NewProjectDirNotEmpty);
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
    if let Err(err) = opts
        .template
        .get_dir()
        .extract(Path::new(&opts.dest_dir).join(&opts.project_name))
    {
        log::error!(
            "Can't unpack vertigo stub to {}: {}",
            target_path.to_string_lossy(),
            err
        );
        return Err(ErrorCode::NewProjectCantUnpackStub);
    };

    // Process all Cargo.toml_ files recursively
    // (cargo packaging does not permit adding second Cargo.toml file)
    process_cargo_toml_files(&target_path, &opts.project_name)?;

    Ok(())
}

/// Find all Cargo.toml_ files, replace "my_app" with package_name, and save as Cargo.toml
fn process_cargo_toml_files(dir: &Path, package_name: &str) -> Result<(), ErrorCode> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml_" {
            let path = entry.path();
            // Read the content
            let content = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(err) => {
                    log::error!("Can't read {}: {}", path.display(), err);
                    return Err(ErrorCode::NewProjectCantCreateCargoToml);
                }
            };

            // Replace my_app with package_name
            let new_content = content.replace("my_app", package_name);

            // Write to Cargo.toml in the same directory
            if let Some(parent) = path.parent() {
                let cargo_toml_path = parent.join("Cargo.toml");
                if let Err(err) = fs::write(&cargo_toml_path, new_content) {
                    log::error!("Can't write to {}: {}", cargo_toml_path.display(), err);
                    return Err(ErrorCode::NewProjectCanWriteToCargoToml);
                }
            }

            // Remove the original Cargo.toml_ file
            if let Err(err) = fs::remove_file(path) {
                log::error!("Can't remove {}: {}", path.display(), err);
                return Err(ErrorCode::NewProjectCantCreateCargoToml);
            }
        }
    }

    Ok(())
}
