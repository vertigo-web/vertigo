use std::path::PathBuf;
use serde::Deserialize;

use crate::command::CommandRun;

#[derive(Clone, Debug, Deserialize)]
pub struct Workspace {
    packages: Vec<Package>,
    target_directory: String,
    workspace_members: Vec<String>,
    workspace_root: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    id: String,
    name: String,
    manifest_path: String,
    targets: Vec<Target>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Target {
    kind: Vec<String>,
}

impl Workspace {
    pub fn infer_package_name(&self) -> Option<String> {
        for member_id in &self.workspace_members {
            if let Some(package) = self.packages
                .iter()
                .find(|package| &package.id == member_id)
            {
                if package.is_cdylib() {
                    return Some(package.name.clone())
                }
            }
        }
        None
    }

    pub fn find_package_path(&self, package_name: &str) -> Option<PathBuf> {
        self.packages.iter()
            .find(|package| package.name == package_name)
            .map(|package| package.manifest_path.clone().into())
    }

    pub fn get_target_dir(&self) -> PathBuf {
        self.target_directory.clone().into()
    }

    pub fn get_root_dir(&self) -> &str {
        &self.workspace_root
    }
}

impl Package {
    pub fn is_cdylib(&self) -> bool {
        self.targets.iter()
            .any(|target|
                target.kind.iter()
                    .any(|kind| kind == "cdylib")
            )
    }
}

pub fn get_workspace() -> Result<Workspace, String> {
    let metadata = CommandRun::new("cargo")
        .add_param("metadata")
        .add_param("--format-version=1")
        .output();

    match serde_json::from_str::<Workspace>(&metadata) {
        Ok(mut ws) => {
            // Retain only local packages to keep object thin
            ws.packages.retain(|package| ws.workspace_members.contains(&package.id));
            Ok(ws)
        },
        Err(err) => {
            Err(format!("Can't load workspace: {err}"))
        }
    }
}
