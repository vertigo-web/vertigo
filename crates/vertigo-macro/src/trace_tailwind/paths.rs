use std::error::Error;
use std::path::PathBuf;

use crate::get_target_dir::get_target_dir;

pub(crate) fn get_tailwind_classes_dir() -> Result<PathBuf, Box<dyn Error>> {
    let dir = get_target_dir().join("tailwind");
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Can't create tailwind subdirectory: {}", err))?;
    Ok(dir)
}

pub(crate) fn get_tailwind_classes_file_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(get_tailwind_classes_dir()?.join("classes.html"))
}

pub(crate) fn get_tailwind_binary_path() -> Result<PathBuf, Box<dyn Error>> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let binary_name = match (os, arch) {
        ("windows", "x86_64") => "tailwindcss-windows-x64.exe",
        ("windows", "aarch64") => "tailwindcss-windows-arm64.exe",
        ("macos", "x86_64") => "tailwindcss-macos-x64",
        ("macos", "aarch64") => "tailwindcss-macos-arm64",
        ("linux", "x86_64") => "tailwindcss-linux-x64",
        ("linux", "aarch64") => "tailwindcss-linux-arm64",
        _ => {
            return Err(format!(
                "Unsupported OS or architecture for standalone Tailwind CLI: {os}-{arch}"
            )
            .into());
        }
    };

    let tailwind_version =
        std::env::var("VERTIGO_TAILWIND_VERSION").unwrap_or_else(|_| "v4.2.0".to_string());

    let cache_dir = get_target_dir()
        .join("tailwind_cli_cache")
        .join(&tailwind_version);
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    let filename = if os == "windows" {
        "tailwindcss.exe"
    } else {
        "tailwindcss"
    };
    let executable_path = cache_dir.join(filename);

    if !executable_path.exists() {
        let url = format!(
            "https://github.com/tailwindlabs/tailwindcss/releases/download/{tailwind_version}/{binary_name}"
        );

        println!(
            "Downloading Tailwind CSS CLI from {} to {}",
            url,
            executable_path.display()
        );

        let response = minreq::get(&url).send()?;
        if response.status_code != 200 {
            return Err(format!(
                "Failed to download Tailwind CLI: HTTP {}",
                response.status_code
            )
            .into());
        }

        let mut file = std::fs::File::create(&executable_path)?;
        std::io::copy(&mut response.as_bytes(), &mut file)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&executable_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&executable_path, perms)?;
        }
    }

    Ok(executable_path)
}
