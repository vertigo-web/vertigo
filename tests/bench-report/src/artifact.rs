//! What a run shipped, and how big it was.
//!
//! The suites already record how long things took and what work was done. This is the other
//! half: the bytes a visitor downloads. Recorded per run rather than per row, and compared
//! exactly rather than through a noise band - a file is the size it is.

use std::{collections::BTreeMap, io::Write, path::Path};

use crate::{BenchResult, Ctx};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Artifact {
    /// Join key across runs, e.g. `ssr-bench wasm`.
    pub name: String,
    /// The hashed filename vertigo wrote.
    ///
    /// Content-addressed, so it changes whenever the bytes do. That makes *same size, different
    /// content* a thing the comparison can notice; a size table alone hides it.
    pub file: String,
    /// On disk, after `wasm-opt -Os` - what `vertigo build` ships.
    pub bytes: u64,
    /// What actually crosses the wire.
    pub gzip_bytes: u64,
    /// Wasm sections by name. Empty for anything that is not wasm.
    pub parts: BTreeMap<String, u64>,
}

impl Artifact {
    /// Every artifact `vertigo build` wrote into `dest_dir`, read from its `index.json`.
    ///
    /// That file names exactly the two things vertigo produced, under a
    /// `%%VERTIGO_PUBLIC_BUILD_PATH%%/` placeholder the server substitutes at request time. The
    /// files themselves sit in `dest_dir` under those basenames.
    pub fn scan(label: &str, dest_dir: &str) -> BenchResult<Vec<Artifact>> {
        let dir = Path::new(dest_dir);
        let index_path = dir.join("index.json");

        let text = std::fs::read_to_string(&index_path).ctx(format!(
            "reading {} - the build did not run, or ran somewhere else",
            index_path.display()
        ))?;
        let index: serde_json::Value =
            serde_json::from_str(&text).ctx(format!("parsing {}", index_path.display()))?;

        let mut out = Vec::new();

        // `wasm` first: it is the one anybody reads.
        for (key, suffix) in [("wasm", "wasm"), ("run_js", "js")] {
            let Some(declared) = index.get(key).and_then(serde_json::Value::as_str) else {
                continue;
            };

            let file = basename(declared);
            let path = dir.join(&file);
            let bytes = std::fs::read(&path).ctx(format!("reading {}", path.display()))?;

            let gzip_bytes = gzip_size(&bytes).ctx(format!("compressing {}", path.display()))?;

            out.push(Artifact {
                name: format!("{label} {suffix}"),
                file,
                bytes: bytes.len() as u64,
                gzip_bytes,
                parts: wasm_sections(&bytes),
            });
        }

        Ok(out)
    }

    /// Whether this artifact and another describe the same bytes.
    ///
    /// By the hashed filename rather than by size, which is the point of having it.
    pub fn same_content(&self, other: &Artifact) -> bool {
        self.file == other.file
    }
}

/// The part after the last `/`, which is where the file actually is.
fn basename(declared: &str) -> String {
    declared.rsplit('/').next().unwrap_or(declared).to_string()
}

/// Gzipped length at a **fixed** level.
///
/// Pinned because an absolute compressed size only means anything if every run used the same
/// setting, and `best` because a precompressed static asset is what a server hands out.
fn gzip_size(bytes: &[u8]) -> BenchResult<u64> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());

    encoder.write_all(bytes).ctx("gzipping")?;

    Ok(encoder.finish().ctx("finishing the gzip stream")?.len() as u64)
}

/// Section sizes of a wasm module, or nothing at all if this is not one.
///
/// The format is a `\0asm` magic plus a version word, then a flat sequence of
/// `id: u8, size: uleb128, payload`. Payload sizes are what is reported, so they sum to slightly
/// less than the file - the header and each section's own id and length byte are not in any
/// section. That is the same convention `wasm-objdump -h` prints.
///
/// Anything that does not start with the magic - the JS, or a truncated file - yields an empty
/// map rather than an error, so both artifacts take the same path.
fn wasm_sections(bytes: &[u8]) -> BTreeMap<String, u64> {
    const NAMES: [&str; 13] = [
        "custom",
        "type",
        "import",
        "function",
        "table",
        "memory",
        "global",
        "export",
        "start",
        "elem",
        "code",
        "data",
        "datacount",
    ];

    let mut parts = BTreeMap::new();

    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        return parts;
    }

    let mut at = 8;

    while at < bytes.len() {
        let Some(id) = bytes.get(at).copied() else {
            break;
        };
        at += 1;

        let Some((size, read)) = uleb128(bytes, at) else {
            break;
        };
        at += read;

        // A length that runs off the end means the file is truncated or not what it claimed to
        // be. Stop and report what was read rather than guessing at the rest.
        let Some(end) = at
            .checked_add(size as usize)
            .filter(|end| *end <= bytes.len())
        else {
            break;
        };

        let name = NAMES.get(id as usize).copied().unwrap_or("unknown");
        *parts.entry(name.to_string()).or_insert(0) += size;

        at = end;
    }

    parts
}

/// An unsigned LEB128 at `at`, and how many bytes it took.
fn uleb128(bytes: &[u8], at: usize) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0;
    let mut read = 0;

    loop {
        let byte = bytes.get(at + read).copied()?;
        read += 1;

        value |= u64::from(byte & 0x7f) << shift;

        if byte & 0x80 == 0 {
            return Some((value, read));
        }

        shift += 7;

        // Ten groups of seven bits is more than a u64 holds; anything longer is malformed.
        if shift >= 64 {
            return None;
        }
    }
}
