//! One run: its rows, the table it prints, and the file it writes.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{BenchResult, Ctx, SCHEMA, artifact::Artifact, meta::Meta, stats::Stats, suite::Suite};

// --------------------------------------------------------------------------------------
// rows
// --------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Row {
    /// What the row is of - a route, a workload. Also the join key when two runs are compared,
    /// so it has to be stable across runs and unique within one.
    pub key: String,
    /// How many observations each of the metrics was reduced from.
    pub samples: usize,
    pub metrics: BTreeMap<String, Stats>,
    /// The deterministic half of the row. A change here is a change in *what* was measured,
    /// which is a different thing from a change in how fast it was, and has to be read first.
    pub counters: BTreeMap<String, i64>,
}

impl Row {
    pub fn new(key: impl Into<String>) -> Row {
        Row {
            key: key.into(),
            ..Row::default()
        }
    }

    pub fn samples(mut self, samples: usize) -> Row {
        self.samples = samples;
        self
    }

    /// Records a metric from its raw observations.
    ///
    /// No observations records nothing, rather than recording zero. Not every row of a suite is
    /// necessarily measured the same way - the hydration suite's demo rows have no probe and so
    /// no marks at all - and a zero in the table reads as "instant" rather than as "not
    /// measured here".
    pub fn metric(mut self, name: &str, samples: Vec<f64>) -> Row {
        if samples.is_empty() {
            return self;
        }

        self.metrics.insert(name.to_string(), Stats::of(samples));
        self
    }

    /// Records a metric that arrived already reduced.
    pub fn metric_stats(mut self, name: &str, stats: Stats) -> Row {
        self.metrics.insert(name.to_string(), stats);
        self
    }

    /// Records `batch_ms` and the `per_op_us` derived from it.
    ///
    /// The per-operation cost is the batch divided by a constant, so it is derived **per
    /// sample** rather than from the batch median: that gives it a real spread instead of one
    /// number repeated once per batch.
    ///
    /// Shared by the reactive and DOM suites, which measure the same shape of thing in the same
    /// units - and which are meant to be read against each other, so the two spellings of this
    /// had better not drift.
    pub fn batch(self, batch_ms: Vec<f64>, iters: u64) -> Row {
        let per_op: Vec<f64> = batch_ms
            .iter()
            .map(|sample| sample * 1000.0 / iters.max(1) as f64)
            .collect();

        self.samples(batch_ms.len())
            .metric("batch_ms", batch_ms)
            .metric("per_op_us", per_op)
            .counter("iters", iters as i64)
    }

    pub fn counter(mut self, name: &str, value: i64) -> Row {
        self.counters.insert(name.to_string(), value);
        self
    }

    pub fn counters(mut self, values: impl IntoIterator<Item = (String, i64)>) -> Row {
        self.counters.extend(values);
        self
    }

    pub fn get_metric(&self, name: &str) -> Stats {
        self.metrics.get(name).copied().unwrap_or_default()
    }

    pub fn get_counter(&self, name: &str) -> i64 {
        self.counters.get(name).copied().unwrap_or(0)
    }
}

// --------------------------------------------------------------------------------------
// the run
// --------------------------------------------------------------------------------------

pub struct Run {
    pub suite: &'static Suite,
    pub meta: Meta,
    pub label: String,
    /// Extra header lines, printed under `git` and not otherwise interpreted. Where a suite
    /// says what it built and how big it was.
    pub header: Vec<String>,
    /// How the samples were taken. Every entry is also a field the comparison guards, so a
    /// baseline taken with a different sample count says so rather than being subtracted.
    pub harness: BTreeMap<String, serde_json::Value>,
    /// Anything else worth storing but not worth guarding.
    ///
    /// Separate from [`Self::harness`] because the guard's job is to catch a difference that
    /// makes two runs incomparable, and these are differences that are the point of comparing:
    /// the size of the wasm a suite built is exactly what a change is expected to move, so
    /// warning about it would fire on every legitimate comparison.
    pub notes: BTreeMap<String, serde_json::Value>,
    /// What this run shipped. Compared exactly rather than through a noise band - see
    /// [`crate::Artifact`].
    pub artifacts: Vec<Artifact>,
    pub rows: Vec<Row>,
}

impl Run {
    pub fn new(suite: &'static Suite, meta: Meta, label: String) -> Run {
        Run {
            suite,
            meta,
            label,
            header: Vec::new(),
            harness: BTreeMap::new(),
            notes: BTreeMap::new(),
            artifacts: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn header_line(mut self, line: impl Into<String>) -> Run {
        self.header.push(line.into());
        self
    }

    pub fn harness(mut self, name: &str, value: impl Into<serde_json::Value>) -> Run {
        self.harness.insert(name.to_string(), value.into());
        self
    }

    pub fn note(mut self, name: &str, value: impl Into<serde_json::Value>) -> Run {
        self.notes.insert(name.to_string(), value.into());
        self
    }

    /// Adds to whatever is already recorded, because a suite that builds two apps calls this
    /// once per build.
    pub fn artifacts(mut self, artifacts: Vec<Artifact>) -> Run {
        self.artifacts.extend(artifacts);
        self
    }

    pub fn rows(mut self, rows: Vec<Row>) -> Run {
        self.rows = rows;
        self
    }

    fn headline(&self, row: &Row) -> Stats {
        row.get_metric(self.suite.headline)
    }

    /// The counters that get a column.
    ///
    /// Only the ones the suite declared. A row may carry more - the DOM suite records a counter
    /// per command variant, and there are thirteen of those - and those are still stored and
    /// still compared; they are just not worth thirteen columns of mostly `-`. A suite that
    /// wants them read prints them itself, the way it chooses to group them.
    fn counter_columns(&self) -> &'static [&'static str] {
        self.suite.counters
    }
}

// --------------------------------------------------------------------------------------
// the table
// --------------------------------------------------------------------------------------

/// Width of the key column. Wide enough for `bench[off] /mismatch-depth`, which is the longest
/// key any of the four suites produces.
///
/// Shared with [`crate::compare`]: the run table and the delta table are read one under the
/// other, so two declarations that drifted apart would silently misalign them.
pub(crate) const KEY: usize = 28;

impl Run {
    pub fn print(&self) {
        let suite = self.suite;
        let meta = &self.meta;

        println!();
        println!("{}", suite.title);
        println!(
            "  host      : {}, {}, {} cpus{}",
            meta.os,
            meta.cpu_model,
            meta.cpus,
            if meta.governor.is_empty() {
                String::new()
            } else {
                format!(", governor={}", meta.governor)
            }
        );
        println!("  rustc     : {}", meta.rustc);
        if !meta.user_agent.is_empty() {
            println!("  browser   : {}", meta.user_agent);
        }
        println!("  git       : {}", meta.describe());
        if self.label != meta.label_sha() {
            println!("  label     : {}", self.label);
        }
        for line in &self.header {
            println!("  {line}");
        }
        if !self.harness.is_empty() {
            let described: Vec<String> = self
                .harness
                .iter()
                .map(|(name, value)| format!("{name} {}", render(value)))
                .collect();
            println!("  harness   : {}", described.join(", "));
        }
        println!();

        self.print_artifacts();
        self.print_headline();
        self.print_metrics();
        self.print_counters();
        self.print_unstable();

        println!();
    }

    /// What this run shipped, before anything about how fast it was.
    fn print_artifacts(&self) {
        if self.artifacts.is_empty() {
            return;
        }

        println!("  artifacts");

        for artifact in &self.artifacts {
            print!(
                "    {:<22} {:>12}   gz {:>11}",
                artifact.name,
                bytes(artifact.bytes),
                bytes(artifact.gzip_bytes),
            );

            // Sections, biggest first: the one that moved is nearly always `code`, and a reader
            // should not have to look for it. Everything under a kilobyte is summed into
            // `other` rather than given a column - a wasm module has a dozen sections and most
            // of them are a handful of bytes. The full map is in the JSON either way.
            let mut parts: Vec<(&String, u64)> = artifact
                .parts
                .iter()
                .map(|(name, size)| (name, *size))
                .collect();
            parts.sort_by_key(|(_, size)| std::cmp::Reverse(*size));

            let small: u64 = parts
                .iter()
                .filter(|(_, size)| *size < 1024)
                .map(|(_, size)| *size)
                .sum();

            for (name, size) in parts.iter().filter(|(_, size)| *size >= 1024) {
                print!("  {name} {}", bytes(*size));
            }

            if small > 0 {
                print!("  other {}", bytes(small));
            }

            println!();
        }

        println!();
    }

    fn print_headline(&self) {
        let suite = self.suite;
        let unit = suite.unit_of(suite.headline);

        println!(
            "  {:<KEY$} {:>13} {:>13} {:>13}",
            format!("key ({}, {unit})", suite.headline),
            "min",
            "median",
            "p90",
        );

        for row in &self.rows {
            if !row.metrics.contains_key(suite.headline) {
                println!("  {:<KEY$} {:>13} {:>13} {:>13}", row.key, "-", "-", "-");
                continue;
            }

            let stats = self.headline(row);
            println!(
                "  {:<KEY$} {:>13.3} {:>13.3} {:>13.3}{}",
                row.key,
                stats.min,
                stats.median,
                stats.p90,
                if stats.unstable(suite.instability_ratio, suite.instability_floor) {
                    "  !"
                } else {
                    ""
                }
            );
        }
    }

    /// The remaining metrics, medians only. Skipped when the headline is the only one.
    fn print_metrics(&self) {
        let suite = self.suite;

        if suite.metrics.len() < 2 {
            return;
        }

        println!();
        println!("  medians by metric. For a suite whose metrics are phases of one operation,");
        println!("  the medians of the parts do not sum to the median of the whole.");
        println!();

        print!("  {:<KEY$}", "key");
        for metric in suite.metrics {
            print!(" {:>13}", format!("{} {}", metric.name, metric.unit));
        }
        println!();

        for row in &self.rows {
            print!("  {:<KEY$}", row.key);
            for metric in suite.metrics {
                match row.metrics.get(metric.name) {
                    Some(stats) => print!(" {:>13.3}", stats.median),
                    None => print!(" {:>13}", "-"),
                }
            }
            println!();
        }
    }

    fn print_counters(&self) {
        let columns = self.counter_columns();

        if columns.is_empty() {
            return;
        }

        println!();
        println!("  counters. Deterministic: these are what was measured, not how fast.");
        println!();

        print!("  {:<KEY$}", "key");
        for name in columns {
            print!(" {name:>13}");
        }
        println!();

        for row in &self.rows {
            print!("  {:<KEY$}", row.key);
            for name in columns {
                match row.counters.get(*name) {
                    // A counter a row does not carry is different from one it carries as zero:
                    // hydration publishes no coverage at all when it is switched off, and
                    // printing `0/0` for that reads as a measurement rather than as an absence.
                    Some(value) => print!(" {value:>13}"),
                    None => print!(" {:>13}", "-"),
                }
            }
            println!();
        }
    }

    fn print_unstable(&self) {
        let suite = self.suite;
        let unstable: Vec<&Row> = self
            .rows
            .iter()
            .filter(|row| {
                self.headline(row)
                    .unstable(suite.instability_ratio, suite.instability_floor)
            })
            .collect();

        if unstable.is_empty() {
            return;
        }

        println!();
        for row in unstable {
            let stats = self.headline(row);
            println!(
                "  ! {:<KEY$} median/min = {:.2} - the machine was busy; re-run before quoting this",
                row.key,
                stats.median / stats.min.max(f64::MIN_POSITIVE)
            );
        }
    }
}

/// Digits grouped in threes. A seven-digit byte count is unreadable otherwise, and these are
/// numbers people compare by eye.
pub(crate) fn bytes(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }

    out
}

fn render(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

// --------------------------------------------------------------------------------------
// the file
// --------------------------------------------------------------------------------------

fn stats_json(stats: &Stats) -> serde_json::Value {
    serde_json::json!({ "min": stats.min, "median": stats.median, "p90": stats.p90 })
}

impl Run {
    pub fn to_json(&self, created_at_unix: u64) -> serde_json::Value {
        let meta = &self.meta;

        serde_json::json!({
            "schema": SCHEMA,
            "kind": self.suite.kind,
            "created_at_unix": created_at_unix,
            "label": self.label,
            "git": {
                "short": meta.git_sha, "branch": meta.git_branch, "dirty": meta.git_dirty,
            },
            "toolchain": { "rustc": meta.rustc, "profile": "release" },
            "machine": {
                "os": meta.os, "cpu_model": meta.cpu_model,
                "cpus": meta.cpus, "governor": meta.governor,
                "user_agent": meta.user_agent,
            },
            "harness": self.harness.iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect::<serde_json::Map<_, _>>(),
            "notes": self.notes.iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect::<serde_json::Map<_, _>>(),
            "artifacts": self.artifacts.iter().map(|artifact| serde_json::json!({
                "name": artifact.name,
                "file": artifact.file,
                "bytes": artifact.bytes,
                "gzip_bytes": artifact.gzip_bytes,
                "parts": artifact.parts.iter()
                    .map(|(name, size)| (name.clone(), serde_json::json!(size)))
                    .collect::<serde_json::Map<_, _>>(),
            })).collect::<Vec<_>>(),
            // Declared here as well as in the rows, so a reader knows the intended order and
            // the unit of every column without having to know which suite wrote the file.
            "metrics": self.suite.metrics.iter().map(|metric| serde_json::json!({
                "name": metric.name, "unit": metric.unit,
            })).collect::<Vec<_>>(),
            "headline": self.suite.headline,
            "rows": self.rows.iter().map(|row| serde_json::json!({
                "key": row.key,
                "samples": row.samples,
                "metrics": row.metrics.iter()
                    .map(|(name, stats)| (name.clone(), stats_json(stats)))
                    .collect::<serde_json::Map<_, _>>(),
                "counters": row.counters.iter()
                    .map(|(name, value)| (name.clone(), serde_json::json!(value)))
                    .collect::<serde_json::Map<_, _>>(),
            })).collect::<Vec<_>>(),
        })
    }

    /// Writes `target/bench/<suite>/<label>-<timestamp>.json`.
    ///
    /// Deliberately **not** `latest.json` as well: that file is the default baseline, and a run
    /// is only fit to be one once it has passed its own gates. See [`Run::promote_to_latest`].
    pub fn write(&self, created_at_unix: u64) -> BenchResult<PathBuf> {
        let dir = PathBuf::from("target/bench").join(self.suite.dir);
        std::fs::create_dir_all(&dir).ctx(format!("creating {}", dir.display()))?;

        let body =
            serde_json::to_string_pretty(&self.to_json(created_at_unix)).ctx("serialising")?;

        let path = dir.join(format!("{}-{created_at_unix}.json", self.file_label()));
        std::fs::write(&path, &body).ctx(format!("writing {}", path.display()))?;

        Ok(path)
    }

    /// Points `latest.json` at a run this suite has finished checking.
    ///
    /// Separate from [`Run::write`] and called last, because `latest.json` is what the
    /// `-compare` tasks subtract against by default. A run that failed its own counter gates
    /// measured work the suite says is wrong; promoting it would make the next comparison
    /// subtract one bad run from another, print no COUNTERS CHANGED block, and launder the
    /// regression into the baseline.
    pub fn promote_to_latest(&self, path: &Path) -> BenchResult {
        let Some(dir) = path.parent() else {
            return Err(format!(
                "{} has no directory to write latest.json into",
                path.display()
            )
            .into());
        };

        let body = std::fs::read(path).ctx(format!("re-reading {}", path.display()))?;
        std::fs::write(dir.join("latest.json"), body).ctx("writing latest.json")?;

        println!("  latest.json now points at this run");

        Ok(())
    }

    /// [`Self::label`], reduced to one path component.
    ///
    /// The hydration suite labels its runs with the git branch, and branches are routinely
    /// called `feat/something`. Left alone that turns the filename into a path into a directory
    /// nobody created, and the write fails *after* all the measuring is done. Only the filename
    /// is folded - the label goes into the JSON and the printed header verbatim.
    pub(crate) fn file_label(&self) -> String {
        let folded: String = self
            .label
            .chars()
            .map(|character| match character {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => character,
                _ => '-',
            })
            .collect();

        match folded.trim_matches(['-', '.']).is_empty() {
            true => "run".to_string(),
            false => folded,
        }
    }
}
