//! Reading an earlier run back, and printing the difference.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    BenchResult, Ctx, SCHEMA,
    artifact::Artifact,
    run::{KEY, Run, bytes},
    stats::Stats,
    suite::Suite,
};

#[derive(Default)]
struct BaseRow {
    metrics: BTreeMap<String, Stats>,
    counters: BTreeMap<String, i64>,
}

/// One guarded field: what the baseline held, and where the same thing lives in a run's JSON.
///
/// The pointer is stored rather than reconstructed from the display name. The name is built by
/// joining path segments with `.`, and a harness key is free to contain one - so going back the
/// other way is guesswork that resolves to nothing and drops the guard silently, which is the
/// one thing a guard must not do.
struct Guard {
    /// RFC 6901 pointer into [`Run::to_json`]'s output.
    pointer: String,
    was: String,
}

pub struct Baseline {
    pub path: PathBuf,
    pub label: String,
    pub created_at_unix: u64,
    /// Schema the file was written with. Kept so the comparison can say when it is reading an
    /// older shape through the adapter rather than the current one.
    pub schema: u64,
    /// The things that, if they differ, make the table a comparison of two machines rather than
    /// of two commits. Keyed by the name to print.
    fields: BTreeMap<String, Guard>,
    rows: BTreeMap<String, BaseRow>,
    artifacts: Vec<Artifact>,
}

/// An RFC 6901 pointer built from literal path segments, with `~` and `/` escaped.
pub(crate) fn json_pointer(segments: &[&str]) -> String {
    segments
        .iter()
        .map(|segment| format!("/{}", segment.replace('~', "~0").replace('/', "~1")))
        .collect()
}

impl Baseline {
    pub fn load(path: &Path, suite: &Suite) -> BenchResult<Baseline> {
        let text = std::fs::read_to_string(path).ctx(format!("reading {}", path.display()))?;
        let root: serde_json::Value = serde_json::from_str(&text).ctx("parsing the baseline")?;

        let kind = root
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();

        // A hydration baseline handed to the SSR suite would otherwise join on nothing and
        // print an empty table, which reads as "no rows changed".
        if kind != suite.kind {
            return Err(format!(
                "{} holds a {kind:?} run, this is {:?} - point it at the right file",
                path.display(),
                suite.kind,
            )
            .into());
        }

        let schema = root
            .get("schema")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let rows = match schema {
            SCHEMA => read_rows(&root),
            // The shape the SSR and hydration suites wrote before the four were unified. Read
            // rather than refused: the runs that are expensive to reproduce are exactly the old
            // ones, and a benchmark whose baselines expire is a benchmark nobody runs twice.
            1 => read_rows_schema1(&root),
            other => {
                return Err(format!(
                    "{} was written with schema {other}, this build reads {SCHEMA} and 1 - \
                     re-run the benchmark on that commit rather than comparing against it",
                    path.display()
                )
                .into());
            }
        };

        let mut fields = BTreeMap::new();
        {
            let mut note = |segments: &[&str]| {
                let pointer = json_pointer(segments);
                if let Some(value) = root.pointer(&pointer) {
                    fields.insert(
                        segments.join("."),
                        Guard {
                            pointer,
                            was: value.to_string(),
                        },
                    );
                }
            };
            note(&["machine", "cpu_model"]);
            note(&["machine", "governor"]);
            note(&["machine", "user_agent"]);
            note(&["toolchain", "rustc"]);
        }

        if let Some(harness) = root.get("harness").and_then(serde_json::Value::as_object) {
            for (name, value) in harness {
                fields.insert(
                    format!("harness.{name}"),
                    Guard {
                        pointer: json_pointer(&["harness", name]),
                        was: value.to_string(),
                    },
                );
            }
        }

        Ok(Baseline {
            path: path.to_path_buf(),
            label: root
                .get("label")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("?")
                .to_string(),
            created_at_unix: root
                .get("created_at_unix")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            schema,
            fields,
            rows,
            // Schema 1 predates artifact sizes, so an old baseline reads as none at all rather
            // than as every artifact being new.
            artifacts: read_artifacts(&root),
        })
    }

    #[cfg(test)]
    pub(crate) fn artifacts_for_test(&self) -> &[Artifact] {
        &self.artifacts
    }

    #[cfg(test)]
    pub(crate) fn row_metrics(&self, key: &str) -> Option<&BTreeMap<String, Stats>> {
        self.rows.get(key).map(|row| &row.metrics)
    }

    #[cfg(test)]
    pub(crate) fn row_counters(&self, key: &str) -> Option<&BTreeMap<String, i64>> {
        self.rows.get(key).map(|row| &row.counters)
    }
}

fn read_stats(value: &serde_json::Value) -> Stats {
    let number = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
    };
    Stats {
        min: number("min"),
        median: number("median"),
        p90: number("p90"),
    }
}

fn read_map<T>(
    value: Option<&serde_json::Value>,
    convert: impl Fn(&serde_json::Value) -> Option<T>,
) -> BTreeMap<String, T> {
    value
        .and_then(serde_json::Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(name, value)| Some((name.clone(), convert(value)?)))
                .collect()
        })
        .unwrap_or_default()
}

/// Numbers and booleans both; the hydration suite's `rootFound` is a flag that is worth
/// noticing when it changes, and 0/1 is how it is stored.
fn as_counter(value: &serde_json::Value) -> Option<i64> {
    match value {
        serde_json::Value::Bool(flag) => Some(i64::from(*flag)),
        serde_json::Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|value| value as i64)),
        _ => None,
    }
}

fn rows_of(root: &serde_json::Value) -> &[serde_json::Value] {
    root.get("rows")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn text_at(row: &serde_json::Value, key: &str) -> String {
    row.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn read_artifacts(root: &serde_json::Value) -> Vec<Artifact> {
    root.get("artifacts")
        .and_then(serde_json::Value::as_array)
        .map(|list| {
            list.iter()
                .map(|value| {
                    let number = |key: &str| {
                        value
                            .get(key)
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0)
                    };

                    Artifact {
                        name: text_at(value, "name"),
                        file: text_at(value, "file"),
                        bytes: number("bytes"),
                        gzip_bytes: number("gzip_bytes"),
                        parts: read_map(value.get("parts"), |part| part.as_u64()),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn read_rows(root: &serde_json::Value) -> BTreeMap<String, BaseRow> {
    rows_of(root)
        .iter()
        .map(|row| {
            (
                text_at(row, "key"),
                BaseRow {
                    metrics: read_map(row.get("metrics"), |value| Some(read_stats(value))),
                    counters: read_map(row.get("counters"), as_counter),
                },
            )
        })
        .collect()
}

/// The pre-unification shape, mapped onto the current one.
///
/// Both old suites wrote a key built from the same fields the current ones build it from, so
/// rows join across the change. What differs is where the numbers lived: phases under
/// `phases_us`/`phases_ms` with the total beside them rather than among them, and counters
/// split across two or three named objects.
fn read_rows_schema1(root: &serde_json::Value) -> BTreeMap<String, BaseRow> {
    rows_of(root)
        .iter()
        .map(|row| {
            let mut metrics: BTreeMap<String, Stats> = BTreeMap::new();
            let mut counters: BTreeMap<String, i64> = BTreeMap::new();

            for name in ["phases_us", "phases_ms"] {
                metrics.extend(read_map(row.get(name), |value| Some(read_stats(value))));
            }
            if let Some(total) = row.get("total_us") {
                metrics.insert("total".to_string(), read_stats(total));
            }

            for name in ["counters", "mutations", "coverage"] {
                counters.extend(read_map(row.get(name), as_counter));
            }
            if let Some(status) = row.get("status").and_then(as_counter) {
                counters.insert("status".to_string(), status);
            }

            let subject = text_at(row, "subject");
            let route = text_at(row, "route");
            let key = match row.get("hydration").and_then(serde_json::Value::as_str) {
                Some(hydration) => format!("{subject} {route} [{hydration}]"),
                None => format!("{subject} {route}"),
            };

            (key, BaseRow { metrics, counters })
        })
        .collect()
}

// --------------------------------------------------------------------------------------
// the comparison
// --------------------------------------------------------------------------------------

/// `+42 825 B (+4.3%)`, signed, exact.
///
/// No noise band, unlike [`delta`]: a file is the size it is, so any difference at all is real.
pub(crate) fn delta_bytes(base: u64, now: u64) -> String {
    let difference = now as i64 - base as i64;

    if difference == 0 {
        return "-".to_string();
    }

    let sign = if difference > 0 { "+" } else { "-" };
    let magnitude = bytes(difference.unsigned_abs());

    if base == 0 {
        return format!("{sign}{magnitude} B");
    }

    let percent = difference as f64 / base as f64 * 100.0;

    format!("{sign}{magnitude} B ({percent:+.1}%)")
}

/// `+12.3%`, or `~` inside the noise band, or `-` when there is nothing to divide by.
///
/// A change has to clear both the relative and the absolute bar. See [`Suite::noise_band_pct`]
/// and [`crate::Metric::floor`].
pub(crate) fn delta(base: f64, now: f64, band_pct: f64, floor: f64) -> String {
    if base <= 0.0 {
        return "-".to_string();
    }

    let percent = (now - base) / base * 100.0;

    if percent.abs() < band_pct || (now - base).abs() < floor {
        "~".to_string()
    } else {
        format!("{percent:+.1}%")
    }
}

impl Run {
    /// Writes this run, then prints a comparison against the baseline if one was asked for.
    ///
    /// One call rather than two, because two orderings matter here and both are easy to get
    /// wrong:
    ///
    /// - the baseline is **read before** anything is written. The `-compare` tasks default it
    ///   to `latest.json`, so a run that wrote first would end up comparing itself against
    ///   itself and printing a wall of `~`;
    /// - the run is **written whatever that read did**. A missing or corrupt baseline is a
    ///   reason to print no comparison, never a reason to throw away the minutes of measuring
    ///   that just happened - the more so because the file that could not be read is often
    ///   precisely the one this write is about to create.
    ///
    /// `latest.json` is deliberately not written here; see [`Run::promote_to_latest`].
    pub fn write_and_compare(&self, created_at_unix: u64) -> BenchResult<PathBuf> {
        let baseline = self.baseline_if_asked();
        let path = self.write(created_at_unix)?;

        println!("  written to {}", path.display());

        match baseline? {
            Some(baseline) => self.compare_to(&baseline, created_at_unix)?,
            None => {
                println!(
                    "  to compare a later run against this one:\n    {}={}",
                    self.suite.baseline_env,
                    path.display()
                );
                println!();
            }
        }

        Ok(path)
    }

    /// The baseline the suite's variable names, if it names one.
    ///
    /// A named file that is not there is not an error. The `-compare` tasks name
    /// `latest.json` by default, and on a fresh checkout - or the first time a suite is run at
    /// all - there is simply no earlier run to subtract. A file that *is* there and cannot be
    /// read is a different matter and does fail: it means the baseline was named wrongly, or
    /// is truncated, or belongs to another suite, and silently comparing against nothing would
    /// read as "nothing changed".
    fn baseline_if_asked(&self) -> BenchResult<Option<Baseline>> {
        let Ok(path) = std::env::var(self.suite.baseline_env) else {
            return Ok(None);
        };

        let path = path.trim();

        if path.is_empty() {
            return Ok(None);
        }

        let path = Path::new(path);

        if !path.exists() {
            println!();
            println!(
                "  {} names {}, which does not exist yet - nothing to compare against.",
                self.suite.baseline_env,
                path.display()
            );
            return Ok(None);
        }

        Baseline::load(path, self.suite).map(Some)
    }

    pub fn compare(&self, path: &Path, created_at_unix: u64) -> BenchResult {
        let baseline = Baseline::load(path, self.suite)?;

        self.compare_to(&baseline, created_at_unix)
    }

    fn compare_to(&self, baseline: &Baseline, created_at_unix: u64) -> BenchResult {
        println!();
        println!("compared against {}", baseline.path.display());
        println!(
            "  baseline : {} ({})",
            baseline.label,
            age(baseline.created_at_unix, created_at_unix)
        );
        println!("  current  : {}", self.label);
        if baseline.schema != SCHEMA {
            println!(
                "  note     : baseline is schema {}, read through the compatibility adapter",
                baseline.schema
            );
        }

        self.print_guard(baseline, created_at_unix);
        self.print_artifact_deltas(baseline);
        self.print_deltas(baseline);

        Ok(())
    }

    /// What changed about the bytes that ship.
    ///
    /// No noise band, unlike everything below it: a file is the size it is, so any difference at
    /// all is real and is printed exactly.
    fn print_artifact_deltas(&self, baseline: &Baseline) {
        if self.artifacts.is_empty() && baseline.artifacts.is_empty() {
            return;
        }

        println!();
        println!("  artifacts (exact - sizes are deterministic, so there is no noise band here)");

        if baseline.artifacts.is_empty() {
            println!("    the baseline carried no artifact sizes, so there is nothing to subtract");
            return;
        }

        let mut said_something = false;

        for artifact in &self.artifacts {
            let Some(was) = baseline
                .artifacts
                .iter()
                .find(|other| other.name == artifact.name)
            else {
                println!(
                    "    {:<22} {:>12}   only in this run",
                    artifact.name,
                    bytes(artifact.bytes)
                );
                said_something = true;
                continue;
            };

            let grew = artifact.bytes != was.bytes;
            let rebuilt = !artifact.same_content(was);

            if !grew && !rebuilt {
                continue;
            }

            said_something = true;

            if grew {
                println!(
                    "    {:<22} {:>12}   {:>14}   gz {}",
                    artifact.name,
                    bytes(artifact.bytes),
                    delta_bytes(was.bytes, artifact.bytes),
                    delta_bytes(was.gzip_bytes, artifact.gzip_bytes),
                );
            } else {
                // Same size, different bytes. A size table alone hides this, and it is exactly
                // the case where a reader would otherwise conclude nothing had changed.
                println!(
                    "    {:<22} {:>12}   unchanged in size, but rebuilt: {} -> {}",
                    artifact.name,
                    bytes(artifact.bytes),
                    was.file,
                    artifact.file,
                );
            }

            let mut names: Vec<&String> = artifact.parts.keys().chain(was.parts.keys()).collect();
            names.sort();
            names.dedup();

            for name in names {
                let before = was.parts.get(name).copied().unwrap_or(0);
                let after = artifact.parts.get(name).copied().unwrap_or(0);

                if before != after {
                    println!(
                        "      {:<20} {:>12}   {:>14}",
                        name,
                        bytes(after),
                        delta_bytes(before, after)
                    );
                }
            }
        }

        for was in &baseline.artifacts {
            if !self.artifacts.iter().any(|mine| mine.name == was.name) {
                println!(
                    "    {:<22} {:>12}   only in the baseline",
                    was.name,
                    bytes(was.bytes)
                );
                said_something = true;
            }
        }

        if !said_something {
            println!("    identical - both runs shipped the same bytes");
        }
    }

    /// Whatever differs between the two runs that is not the code.
    fn print_guard(&self, baseline: &Baseline, created_at_unix: u64) {
        let mine = self.to_json(created_at_unix);

        let mismatched: Vec<(&String, &String, String)> = baseline
            .fields
            .iter()
            .filter_map(|(name, guard)| {
                // A field the baseline guarded and this run does not record at all is still a
                // difference, and the interesting direction: it means the suite stopped saying
                // something it used to. Reported rather than skipped.
                let now = match mine.pointer(&guard.pointer) {
                    Some(value) => value.to_string(),
                    None => "(not recorded)".to_string(),
                };
                (now != guard.was).then_some((name, &guard.was, now))
            })
            .collect();

        if mismatched.is_empty() {
            return;
        }

        println!();
        println!("  WARNING - these differ, so the table below is not a comparison of the code:");
        for (name, was, now) in mismatched {
            println!("    {name:<22} {was}  ->  {now}");
        }
    }

    fn print_deltas(&self, baseline: &Baseline) {
        let suite = self.suite;

        println!();
        print!(
            "  {:<KEY$} {:>11} {:>11} {:>9}",
            "key", "base med", "this med", "delta"
        );
        for metric in suite.metrics {
            print!(" {:>13}", metric.name);
        }
        println!();

        let mut counter_changes: Vec<String> = Vec::new();
        let mut only_here: Vec<String> = Vec::new();

        for row in &self.rows {
            let Some(base) = baseline.rows.get(&row.key) else {
                only_here.push(row.key.clone());
                continue;
            };

            let headline = row.get_metric(suite.headline);
            let base_headline = base
                .metrics
                .get(suite.headline)
                .copied()
                .unwrap_or_default();
            let floor = suite.floor_of(suite.headline);

            // When the median and the min both moved meaningfully but in opposite directions,
            // one of the two runs was disturbed and the row is not a result. Both have to be
            // meaningful: a significant median against a min that merely jittered the other way
            // is the ordinary case, not a warning.
            let median_delta = headline.median - base_headline.median;
            let min_delta = headline.min - base_headline.min;
            let disagree = median_delta.signum() != min_delta.signum()
                && delta(
                    base_headline.median,
                    headline.median,
                    suite.noise_band_pct,
                    floor,
                ) != "~"
                && delta(base_headline.min, headline.min, suite.noise_band_pct, floor) != "~";

            print!(
                "  {:<KEY$} {:>11.3} {:>11.3} {:>9}{}",
                row.key,
                base_headline.median,
                headline.median,
                delta(
                    base_headline.median,
                    headline.median,
                    suite.noise_band_pct,
                    floor
                ),
                if disagree { "!" } else { " " }
            );
            for metric in suite.metrics {
                let was = base.metrics.get(metric.name).copied().unwrap_or_default();
                print!(
                    " {:>13}",
                    delta(
                        was.median,
                        row.get_metric(metric.name).median,
                        suite.noise_band_pct,
                        metric.floor
                    )
                );
            }
            println!();

            for line in counter_diff(&row.key, &base.counters, &row.counters) {
                counter_changes.push(line);
            }
        }

        let here: Vec<&String> = self.rows.iter().map(|row| &row.key).collect();
        let only_there: Vec<&String> = baseline
            .rows
            .keys()
            .filter(|key| !here.contains(key))
            .collect();

        if !only_here.is_empty() {
            println!();
            println!("  only in this run : {}", only_here.join(", "));
        }
        if !only_there.is_empty() {
            println!(
                "  only in baseline : {}",
                only_there
                    .iter()
                    .map(|key| key.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if counter_changes.is_empty() {
            println!();
            println!("  counters identical - both runs measured the same work, so the timings");
            println!("  above are about how it was done rather than about what was done.");
        } else {
            println!();
            println!("  COUNTERS CHANGED - read this before the timings. The work itself is");
            println!("  different, so a row that got slower may simply have got bigger:");
            for line in counter_changes {
                println!("{line}");
            }
        }
        println!();
    }
}

/// One line per counter that moved, naming it. A row whose command count changed and a row
/// whose hydration coverage changed are different findings and should not share a message.
fn counter_diff(
    key: &str,
    base: &BTreeMap<String, i64>,
    now: &BTreeMap<String, i64>,
) -> Vec<String> {
    let mut names: Vec<&String> = base.keys().chain(now.keys()).collect();
    names.sort();
    names.dedup();

    names
        .into_iter()
        .filter_map(|name| {
            let was = base.get(name);
            let is = now.get(name);

            if was == is {
                return None;
            }

            let show = |value: Option<&i64>| match value {
                Some(value) => value.to_string(),
                None => "-".to_string(),
            };

            Some(format!(
                "    {key:<KEY$} {name:<16} {} -> {}",
                show(was),
                show(is)
            ))
        })
        .collect()
}

fn age(then: u64, now: u64) -> String {
    if then == 0 || now <= then {
        return "age unknown".to_string();
    }

    let seconds = now - then;
    match seconds {
        0..=3_599 => format!("{} min ago", seconds / 60),
        3_600..=86_399 => format!("{} h ago", seconds / 3_600),
        _ => format!("{} days ago", seconds / 86_400),
    }
}
