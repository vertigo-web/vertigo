//! Paths of the machine a release is built on, kept out of its wasm.
//!
//! rustc writes the path of a source file into the binary wherever code may panic, logs or asks
//! for `file!()`. For a crate from the registry, a path dependency outside the workspace and
//! the standard library's generic code these are absolute paths, as a rule under the home
//! directory of whoever builds:
//! `/home/jane/.cargo/registry/src/index.crates.io-…/chrono-0.4.42/src/format/parse.rs`.
//! A wasm is sent to every visitor, so a release build maps those prefixes away with rustc's
//! `--remap-path-prefix`.
//!
//! The flags have to come on top of the ones the project gives rustc itself, and cargo reads
//! those from one place only, the first there is of `CARGO_ENCODED_RUSTFLAGS`, `RUSTFLAGS`,
//! `target.<triple>.rustflags` and `build.rustflags`. Flags given in a later place are
//! ignored, and flags given in an earlier one replace the project's: ours under
//! `target.<triple>.rustflags` would silently drop an `-Copt-level=z` the project has under
//! `build.rustflags`. So cargo is asked which place it reads ([`flags_source`]), and the flags
//! are appended there.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::commons::command::CommandRun;

use super::cargo_build::{TARGET, inherited_package_env};

/// What separates the flags in `CARGO_ENCODED_RUSTFLAGS`.
const ENCODED_SEPARATOR: &str = "\x1f";

/// The flag the probe gives rustc under `build.rustflags`, to see whether rustc gets it.
const PROBE_CFG: &str = "vertigo_reads_build_rustflags";

/// The place cargo reads the flags for rustc from, for the target.
#[derive(Debug, PartialEq)]
enum FlagsSource {
    /// `CARGO_ENCODED_RUSTFLAGS` of the environment, with its value.
    EncodedEnv(String),
    /// `RUSTFLAGS` of the environment, with its value.
    Env(String),
    /// `target.<triple>.rustflags` and `target.<cfg>.rustflags` of the configuration.
    Target,
    /// `build.rustflags` of the configuration, or no place at all.
    Build,
}

/// The cargo command with flags mapping the machine's paths away, added to the project's own.
/// Left as it is when cargo can't be asked where it reads the flags from: a build with the
/// paths in is better than one with the project's flags gone.
pub(super) fn remap_paths(command: CommandRun, cargo_opts: &[String]) -> CommandRun {
    let flags = flags(&prefixes(
        home_dir().as_deref(),
        cargo_home().as_deref(),
        rust_src()
            .as_ref()
            .map(|(dir, commit)| (dir.as_path(), commit.as_str())),
    ));

    if flags.is_empty() {
        return command;
    }

    let Some(source) = flags_source(Path::new("."), cargo_opts) else {
        log::warn!(
            "Can't tell where cargo reads rustc's flags from, so the paths of this machine \
             stay in the wasm (no --remap-path-prefix)"
        );
        return command;
    };

    match with_flags(&source, &flags) {
        Added::Param(config) => command.add_param("--config").add_param(config),
        Added::Env(value) => command.env("CARGO_ENCODED_RUSTFLAGS", value),
    }
}

/// How the flags get to cargo.
#[derive(Debug, PartialEq)]
enum Added {
    /// The value of a `--config` option: cargo joins a list given there with the one of the
    /// project's configuration.
    Param(String),
    /// The value of `CARGO_ENCODED_RUSTFLAGS`, the project's flags with ours after them.
    Env(String),
}

fn with_flags(source: &FlagsSource, flags: &[String]) -> Added {
    match source {
        FlagsSource::EncodedEnv(old) => {
            let old = old.split(ENCODED_SEPARATOR).filter(|flag| !flag.is_empty());
            Added::Env(join_encoded(old, flags))
        }
        // Encoded, which cargo reads before `RUSTFLAGS`: there a path with a space in it stays
        // one flag
        FlagsSource::Env(old) => Added::Env(join_encoded(old.split_whitespace(), flags)),
        FlagsSource::Target => {
            Added::Param(format!("target.{TARGET}.rustflags={}", toml_list(flags)))
        }
        FlagsSource::Build => Added::Param(format!("build.rustflags={}", toml_list(flags))),
    }
}

fn join_encoded<'a>(old: impl Iterator<Item = &'a str>, flags: &'a [String]) -> String {
    old.chain(flags.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(ENCODED_SEPARATOR)
}

/// A list of strings as TOML writes it, for `--config`.
fn toml_list(items: &[String]) -> String {
    let items = items
        .iter()
        .map(|item| format!("\"{}\"", item.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>();
    format!("[{}]", items.join(","))
}

/// The prefixes to map and what to, the more general first: of the ones a path starts with,
/// rustc takes the last.
fn prefixes(
    home: Option<&Path>,
    cargo_home: Option<&Path>,
    rust_src: Option<(&Path, &str)>,
) -> Vec<(PathBuf, String)> {
    let mut prefixes = Vec::new();

    // Whatever else is under the home directory, f. ex. a path dependency
    if let Some(home) = home {
        prefixes.push((home.to_path_buf(), "~".to_string()));
    }

    if let Some(cargo_home) = cargo_home {
        prefixes.push((
            cargo_home.join("registry").join("src"),
            "cargo/registry".to_string(),
        ));
        prefixes.push((
            cargo_home.join("git").join("checkouts"),
            "cargo/git".to_string(),
        ));
    }

    // As the standard library's own, not generic code has it
    if let Some((dir, commit)) = rust_src {
        prefixes.push((dir.to_path_buf(), format!("/rustc/{commit}")));
    }

    prefixes
}

fn flags(prefixes: &[(PathBuf, String)]) -> Vec<String> {
    prefixes
        .iter()
        // A path that isn't UTF-8 can't go into a flag given through cargo's configuration
        .filter_map(|(from, to)| Some(format!("--remap-path-prefix={}={to}", from.to_str()?)))
        .collect()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
}

fn cargo_home() -> Option<PathBuf> {
    match std::env::var_os("CARGO_HOME") {
        Some(cargo_home) => Some(PathBuf::from(cargo_home)).filter(|path| path.is_absolute()),
        None => home_dir().map(|home| home.join(".cargo")),
    }
}

/// Where rustup keeps the standard library's sources, which generic code of it is compiled
/// from into the app's crates, and the commit of rustc they are of.
fn rust_src() -> Option<(PathBuf, String)> {
    let sysroot = rustc_says(&["--print", "sysroot"])?;
    let version = rustc_says(&["-vV"])?;
    let commit = version
        .lines()
        .find_map(|line| line.strip_prefix("commit-hash: "))?;

    let dir = Path::new(sysroot.trim())
        .join("lib")
        .join("rustlib")
        .join("src")
        .join("rust");

    Some((dir, commit.trim().to_string()))
}

fn rustc_says(args: &[&str]) -> Option<String> {
    let output = Command::new("rustc").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
}

/// Where cargo, run in `dir` with `cargo_opts`, reads rustc's flags from. `None` when it
/// can't be told.
///
/// The environment is read as cargo reads it. Between the two places of the configuration
/// cargo itself decides: it checks an empty crate with a flag under `build.rustflags`, which
/// rustc gets only if no `target.<triple>.rustflags` or `target.<cfg>.rustflags` applies -
/// whatever file, variable or `--config` they come from.
fn flags_source(dir: &Path, cargo_opts: &[String]) -> Option<FlagsSource> {
    if let Ok(flags) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        return Some(FlagsSource::EncodedEnv(flags));
    }

    if let Ok(flags) = std::env::var("RUSTFLAGS") {
        return Some(FlagsSource::Env(flags));
    }

    let probe = std::env::temp_dir().join(format!(
        "vertigo-rustflags-probe-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos())
    ));
    let source = probe_flags_source(dir, cargo_opts, &probe);
    let _ = std::fs::remove_dir_all(&probe);
    source
}

fn probe_flags_source(dir: &Path, cargo_opts: &[String], probe: &Path) -> Option<FlagsSource> {
    std::fs::create_dir_all(probe.join("src")).ok()?;
    std::fs::write(
        probe.join("Cargo.toml"),
        "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
    )
    .ok()?;
    std::fs::write(probe.join("src").join("lib.rs"), "").ok()?;

    let mut command = Command::new("cargo");
    command
        .current_dir(dir)
        .args(["check", "--verbose", "--offline", "--target", TARGET])
        .arg("--manifest-path")
        .arg(probe.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(probe.join("target"))
        .arg("--config")
        .arg(format!("build.rustflags=[\"--cfg={PROBE_CFG}\"]"));

    // The configuration the build itself is given on the command line
    for config in config_opts(cargo_opts) {
        command.arg("--config").arg(config);
    }

    for name in inherited_package_env() {
        command.env_remove(name);
    }

    // Cargo reads these two before the configuration, and `flags_source` asks no probe when
    // there is either: the probe asks of the configuration alone, also in a run of the tests
    // with `RUSTFLAGS` set for the build of the tests themselves
    command
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");

    let output = command.output().ok()?;

    // With `--verbose` cargo tells the rustc command it runs before rustc runs it, so what
    // rustc makes of the empty crate doesn't matter and the target's standard library isn't
    // needed. A configuration cargo can't read stops it before, with no command told.
    let told = String::from_utf8_lossy(&output.stderr);
    let rustc = told
        .lines()
        .find(|line| line.contains("--crate-name probe"))?;
    Some(if rustc.contains(PROBE_CFG) {
        FlagsSource::Build
    } else {
        FlagsSource::Target
    })
}

/// The values of the `--config` options among cargo's options.
fn config_opts(cargo_opts: &[String]) -> Vec<&str> {
    let mut configs = Vec::new();
    let mut opts = cargo_opts.iter();

    while let Some(opt) = opts.next() {
        if opt == "--config" {
            configs.extend(opts.next().map(String::as_str));
        } else if let Some(config) = opt.strip_prefix("--config=") {
            configs.push(config);
        }
    }

    configs
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        Added, FlagsSource, config_opts, flags, prefixes, probe_flags_source, toml_list, with_flags,
    };

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn the_more_general_prefix_comes_first() {
        let prefixes = prefixes(
            Some(Path::new("/home/jane")),
            Some(Path::new("/home/jane/.cargo")),
            Some((
                Path::new("/home/jane/.rustup/toolchains/stable/lib/rustlib/src/rust"),
                "abc123",
            )),
        );

        assert_eq!(
            flags(&prefixes),
            strings(&[
                "--remap-path-prefix=/home/jane=~",
                "--remap-path-prefix=/home/jane/.cargo/registry/src=cargo/registry",
                "--remap-path-prefix=/home/jane/.cargo/git/checkouts=cargo/git",
                "--remap-path-prefix=/home/jane/.rustup/toolchains/stable/lib/rustlib/src/rust=/rustc/abc123",
            ])
        );
    }

    #[test]
    fn what_is_not_known_is_not_mapped() {
        assert_eq!(flags(&prefixes(None, None, None)), Vec::<String>::new());
        assert_eq!(
            flags(&prefixes(None, Some(Path::new("/opt/cargo")), None)),
            strings(&[
                "--remap-path-prefix=/opt/cargo/registry/src=cargo/registry",
                "--remap-path-prefix=/opt/cargo/git/checkouts=cargo/git",
            ])
        );
    }

    #[test]
    fn flags_go_where_cargo_reads_them() {
        let flags = strings(&[
            "--remap-path-prefix=/home/jane=~",
            "--remap-path-prefix=/a b=c",
        ]);
        let list = r#"["--remap-path-prefix=/home/jane=~","--remap-path-prefix=/a b=c"]"#;

        assert_eq!(
            with_flags(&FlagsSource::Build, &flags),
            Added::Param(format!("build.rustflags={list}"))
        );
        assert_eq!(
            with_flags(&FlagsSource::Target, &flags),
            Added::Param(format!("target.wasm32-unknown-unknown.rustflags={list}"))
        );
        // after the project's own, and encoded so that the path with a space stays one flag
        assert_eq!(
            with_flags(&FlagsSource::Env("-Copt-level=z  --cfg x".to_string()), &flags),
            Added::Env(
                "-Copt-level=z\x1f--cfg\x1fx\x1f--remap-path-prefix=/home/jane=~\x1f--remap-path-prefix=/a b=c"
                    .to_string()
            )
        );
        assert_eq!(
            with_flags(&FlagsSource::EncodedEnv("--cfg\x1fa b".to_string()), &flags),
            Added::Env(
                "--cfg\x1fa b\x1f--remap-path-prefix=/home/jane=~\x1f--remap-path-prefix=/a b=c"
                    .to_string()
            )
        );
        assert_eq!(
            with_flags(&FlagsSource::EncodedEnv(String::new()), &strings(&["-a"])),
            Added::Env("-a".to_string())
        );
    }

    #[test]
    fn a_list_as_toml_writes_it() {
        assert_eq!(
            toml_list(&strings(&[r"C:\Users\jane=~", r#"say "hi""#])),
            r#"["C:\\Users\\jane=~","say \"hi\""]"#
        );
    }

    #[test]
    fn config_options_of_cargo() {
        let opts = strings(&[
            "--config",
            "a=1",
            "-Zbuild-std",
            "--config=b=2",
            "--features",
            "x",
            "--config",
        ]);
        assert_eq!(config_opts(&opts), vec!["a=1", "b=2"]);
    }

    /// A project with `config` as its `.cargo/config.toml`, and what the probe tells of it.
    fn probed(name: &str, config: Option<&str>, cargo_opts: &[&str]) -> Option<FlagsSource> {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("vertigo-remap-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        if let Err(err) = std::fs::create_dir_all(dir.join(".cargo")) {
            panic!("creating {}: {err}", dir.display());
        }
        if let Some(config) = config
            && let Err(err) = std::fs::write(dir.join(".cargo").join("config.toml"), config)
        {
            panic!("writing the config: {err}");
        }

        let source = probe_flags_source(&dir, &strings(cargo_opts), &dir.join("probe"));
        let _ = std::fs::remove_dir_all(&dir);
        source
    }

    /// Cargo itself tells, so these need cargo, but not the wasm32 target: cargo tells the
    /// command before rustc, which would need the target, runs it.
    #[test]
    fn cargo_tells_where_it_reads_the_flags_from() {
        assert_eq!(probed("none", None, &[]), Some(FlagsSource::Build));
        assert_eq!(
            probed(
                "build",
                Some("[build]\nrustflags = [\"-Copt-level=z\"]\n"),
                &[]
            ),
            Some(FlagsSource::Build)
        );
        assert_eq!(
            probed(
                "target",
                Some("[target.wasm32-unknown-unknown]\nrustflags = [\"-Copt-level=z\"]\n"),
                &[]
            ),
            Some(FlagsSource::Target)
        );
        assert_eq!(
            probed(
                "cfg",
                Some("[target.'cfg(target_arch = \"wasm32\")']\nrustflags = [\"-Copt-level=z\"]\n"),
                &[]
            ),
            Some(FlagsSource::Target)
        );
        // one for another target is none for this one
        assert_eq!(
            probed(
                "other",
                Some(
                    "[target.'cfg(unix)']\nrustflags = [\"-Copt-level=z\"]\n[build]\nrustflags = [\"-Cdebuginfo=0\"]\n"
                ),
                &[]
            ),
            Some(FlagsSource::Build)
        );
        assert_eq!(
            probed(
                "option",
                None,
                &[
                    "--config",
                    "target.wasm32-unknown-unknown.rustflags=[\"-Copt-level=z\"]"
                ]
            ),
            Some(FlagsSource::Target)
        );
        // a configuration cargo can't read: nothing is told
        assert_eq!(probed("broken", Some("[build\n"), &[]), None);
    }
}
