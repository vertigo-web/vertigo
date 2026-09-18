//! Who ran this, on what, and when.
//!
//! Recorded so that a comparison can refuse to be read as a comparison of two commits when it
//! is really a comparison of two machines.

use std::process::Command;

use crate::suite::Suite;

#[derive(Clone, Debug, Default)]
pub struct Meta {
    pub git_sha: String,
    pub git_branch: String,
    pub git_dirty: bool,
    pub rustc: String,
    pub cpu_model: String,
    pub cpus: usize,
    pub governor: String,
    pub os: String,
    /// Empty for the native suite. The three browser suites fill it in once the session is
    /// open, because which browser ran the page is as much part of the machine as the CPU is.
    pub user_agent: String,
}

impl Meta {
    pub fn collect() -> Meta {
        Meta {
            git_sha: run_out("git", &["rev-parse", "--short", "HEAD"]),
            git_branch: run_out("git", &["rev-parse", "--abbrev-ref", "HEAD"]),
            git_dirty: !run_out("git", &["status", "--porcelain"]).is_empty(),
            rustc: run_out("rustc", &["--version"]),
            cpu_model: cpu_model(),
            cpus: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(0),
            governor: std::fs::read_to_string(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
            )
            .unwrap_or_default()
            .trim()
            .to_string(),
            os: std::env::consts::OS.to_string(),
            user_agent: String::new(),
        }
    }

    /// The commit, which is what a run is usually named after.
    pub fn label_sha(&self) -> String {
        let sha = if self.git_sha.is_empty() {
            "nogit"
        } else {
            self.git_sha.as_str()
        };
        let dirty = if self.git_dirty { "-dirty" } else { "" };
        format!("{sha}{dirty}")
    }

    /// The branch, for a suite whose two runs are two implementations rather than two commits.
    pub fn label_branch(&self) -> String {
        if self.git_branch.is_empty() || self.git_branch == "HEAD" {
            self.label_sha()
        } else {
            self.git_branch.clone()
        }
    }

    /// `fallback`, unless the suite's label variable says otherwise.
    pub fn label_for(&self, suite: &Suite, fallback: String) -> String {
        match std::env::var(suite.label_env) {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => fallback,
        }
    }

    pub(crate) fn describe(&self) -> String {
        let dirty = if self.git_dirty { " (dirty)" } else { "" };
        format!("{}{dirty} on {}", self.git_sha, self.git_branch)
    }
}

fn run_out(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split(':').nth(1))
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}

/// FNV-1a, written out rather than reached for.
///
/// `DefaultHasher` is documented as unstable across Rust releases, which is disqualifying for a
/// value whose whole job is to be compared against one computed by a different build.
pub fn body_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}
