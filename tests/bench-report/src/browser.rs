//! What the two in-browser suites share about driving their app and reading its report.
//!
//! `tests/reactive-bench` and `tests/dom-bench` run the same shape of thing: an app built by
//! `vertigo-bench-support`, loaded once with a `?scale=`, which prints one `|`-separated line
//! per workload into `#bench-report`. The two drivers differ in how many fields that line has
//! and what they mean - but not in the scale, and not in how the sample list at the end of it
//! is read, so those two live here rather than in both.

/// What the app is asked to multiply its iteration counts by.
///
/// Passed through the URL rather than the environment because the app is in the browser, and
/// recorded in the harness block because two runs taken at different scales are not comparable.
pub fn scale() -> f64 {
    std::env::var("VERTIGO_BENCH_SCALE")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|scale: &f64| scale.is_finite() && *scale > 0.0)
        .unwrap_or(1.0)
}

/// The `;`-separated batch times at the end of a report line.
///
/// Panics rather than returning an error, like the rest of the report parsing: the text being
/// read was produced by the suite's own app moments earlier, so anything malformed here is a
/// bug in this repository rather than a condition to handle.
pub fn parse_samples(field: &str, line: &str) -> Vec<f64> {
    let samples: Vec<f64> = field
        .split(';')
        .filter(|sample| !sample.is_empty())
        .map(|sample| {
            sample
                .parse()
                .unwrap_or_else(|_| panic!("malformed sample {sample:?} in {line:?}"))
        })
        .collect();

    assert!(!samples.is_empty(), "no batch samples in {line:?}");

    samples
}
