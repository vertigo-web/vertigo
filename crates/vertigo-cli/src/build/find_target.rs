use std::path::{Path, PathBuf};

use super::wasm_path::WasmPath;

/// Profile the build runs under, resolved from `--release` and from any `--profile` the
/// caller passed through in their own cargo options.
pub struct BuildProfile {
    /// Name as cargo spells it on the command line.
    pub name: String,
    /// Directory cargo writes artifacts into, under `<target-dir>/<triple>`.
    pub dir: String,
    /// Set when the name came from the caller's cargo options, in which case `cargo build`
    /// already receives it and must not also get `--release` - cargo rejects both together.
    pub from_cargo_opts: bool,
}

impl BuildProfile {
    pub fn resolve(release: bool, cargo_opts: &[String]) -> Self {
        if let Some(name) = find_opt_value(cargo_opts, "--profile") {
            return Self {
                dir: profile_dir(name).to_string(),
                name: name.to_string(),
                from_cargo_opts: true,
            };
        }

        let name = if release { "release" } else { "dev" };

        Self {
            name: name.to_string(),
            dir: profile_dir(name).to_string(),
            from_cargo_opts: false,
        }
    }
}

/// Directory cargo writes into for a profile - the built-in ones don't match their names.
fn profile_dir(profile: &str) -> &str {
    match profile {
        "dev" | "test" => "debug",
        "bench" => "release",
        other => other,
    }
}

/// Root of the target directory. A `--target-dir` in the caller's cargo options wins,
/// otherwise we take the path `cargo metadata` reported - that one already accounts for
/// `CARGO_TARGET_DIR` and for `build.target-dir` in cargo config, but not for a flag
/// handed to `cargo build`, which metadata never sees.
pub fn resolve_target_root(ws_target_dir: PathBuf, cargo_opts: &[String]) -> PathBuf {
    match target_dir_opt(cargo_opts) {
        Some(dir) => PathBuf::from(dir),
        None => ws_target_dir,
    }
}

/// `--target-dir` exactly as the caller spelled it, if they passed one at all.
pub fn target_dir_opt(cargo_opts: &[String]) -> Option<&str> {
    find_opt_value(cargo_opts, "--target-dir")
}

/// Value of `--flag value` or `--flag=value` in caller-supplied cargo options.
fn find_opt_value<'a>(cargo_opts: &'a [String], flag: &str) -> Option<&'a str> {
    let mut opts = cargo_opts.iter();

    while let Some(opt) = opts.next() {
        let Some(rest) = opt.strip_prefix(flag) else {
            continue;
        };

        if rest.is_empty() {
            return opts.next().map(String::as_str);
        }

        if let Some(value) = rest.strip_prefix('=') {
            return Some(value);
        }
    }

    None
}

/// Locate the built wasm inside `target_dir`, which must be the path `run_cargo_build`
/// returned - it accounts for `cargo metadata`, `--target-dir` and the selected profile.
pub fn find_wasm_in_target(target_dir: &Path, package_name: &str) -> WasmPath {
    let wasm_file_name = package_name.replace('-', "_");
    WasmPath::new(target_dir.join(format!("{wasm_file_name}.wasm")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn opt_value_separate_and_joined() {
        assert_eq!(
            find_opt_value(&opts(&["--target-dir", "/tmp/out"]), "--target-dir"),
            Some("/tmp/out")
        );
        assert_eq!(
            find_opt_value(&opts(&["--target-dir=/tmp/out"]), "--target-dir"),
            Some("/tmp/out")
        );
    }

    #[test]
    fn opt_value_absent_or_only_a_prefix_match() {
        assert_eq!(find_opt_value(&opts(&["--offline"]), "--profile"), None);
        // Must not match a longer flag that merely starts with the same characters
        assert_eq!(
            find_opt_value(&opts(&["--target-dirty", "x"]), "--target-dir"),
            None
        );
        // Flag given last, with no value following it
        assert_eq!(find_opt_value(&opts(&["--profile"]), "--profile"), None);
    }

    #[test]
    fn profile_defaults_to_release_flag() {
        let profile = BuildProfile::resolve(true, &[]);
        assert_eq!(
            (profile.name.as_str(), profile.dir.as_str()),
            ("release", "release")
        );
        assert!(!profile.from_cargo_opts);

        let profile = BuildProfile::resolve(false, &[]);
        assert_eq!(
            (profile.name.as_str(), profile.dir.as_str()),
            ("dev", "debug")
        );
    }

    #[test]
    fn profile_from_cargo_opts_wins_over_release() {
        let profile = BuildProfile::resolve(true, &opts(&["--profile", "prod"]));
        assert_eq!(
            (profile.name.as_str(), profile.dir.as_str()),
            ("prod", "prod")
        );
        assert!(profile.from_cargo_opts);
    }

    #[test]
    fn builtin_profiles_map_to_their_directories() {
        for (name, dir) in [
            ("dev", "debug"),
            ("test", "debug"),
            ("bench", "release"),
            ("release", "release"),
        ] {
            let profile = BuildProfile::resolve(false, &opts(&["--profile", name]));
            assert_eq!(profile.dir, dir, "profile {name}");
        }
    }

    #[test]
    fn target_root_prefers_the_cargo_opt() {
        let from_metadata = PathBuf::from("/ws/target");

        assert_eq!(
            resolve_target_root(from_metadata.clone(), &[]),
            from_metadata
        );
        assert_eq!(
            resolve_target_root(from_metadata, &opts(&["--target-dir=/tmp/out"])),
            PathBuf::from("/tmp/out")
        );
    }
}
