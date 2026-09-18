//! What is worth pinning: the noise rules, and that an old baseline still reads.

use crate::{
    Artifact, Meta, Metric, Row, Run, Stats, Suite,
    compare::{Baseline, delta, delta_bytes, json_pointer},
    run::bytes,
};

const SUITE: Suite = Suite {
    kind: "vertigo-test-bench",
    dir: "test",
    title: "a suite that exists only here",
    metrics: &[
        Metric {
            name: "total",
            unit: "us",
            floor: 10.0,
        },
        Metric {
            name: "part",
            unit: "us",
            floor: 10.0,
        },
    ],
    headline: "total",
    counters: &["commands"],
    noise_band_pct: 3.0,
    instability_ratio: 1.25,
    instability_floor: 50.0,
    baseline_env: "VERTIGO_TEST_BENCH_BASELINE",
    label_env: "VERTIGO_TEST_BENCH_LABEL",
};

#[test]
fn stats_are_the_min_the_median_and_the_p90() {
    let stats = Stats::of(vec![5.0, 1.0, 4.0, 2.0, 3.0, 10.0, 6.0, 7.0, 8.0, 9.0]);

    assert_eq!(stats.min, 1.0);
    assert_eq!(stats.median, 6.0);
    assert_eq!(stats.p90, 10.0);
}

#[test]
fn an_empty_sample_set_is_zero_rather_than_a_panic() {
    assert_eq!(Stats::of(vec![]), Stats::default());
}

#[test]
fn a_delta_has_to_clear_both_bars() {
    // Big relatively, small absolutely: three microseconds becoming four is not a finding.
    assert_eq!(delta(3.0, 4.0, 3.0, 10.0), "~");
    // Big absolutely, small relatively.
    assert_eq!(delta(10_000.0, 10_100.0, 3.0, 10.0), "~");
    // Both.
    assert_eq!(delta(1_000.0, 1_500.0, 3.0, 10.0), "+50.0%");
    // Nothing to divide by.
    assert_eq!(delta(0.0, 12.0, 3.0, 10.0), "-");
}

#[test]
fn instability_needs_a_ratio_and_a_size() {
    let jittery_but_small = Stats {
        min: 1.0,
        median: 2.0,
        p90: 3.0,
    };
    assert!(!jittery_but_small.unstable(1.25, 50.0));

    let genuinely_disturbed = Stats {
        min: 1_000.0,
        median: 2_000.0,
        p90: 3_000.0,
    };
    assert!(genuinely_disturbed.unstable(1.25, 50.0));
}

fn sample_run() -> Run {
    Run::new(&SUITE, Meta::default(), "under-test".to_string())
        .harness("samples", 10)
        .rows(vec![
            Row::new("app /one")
                .samples(3)
                .metric("total", vec![100.0, 110.0, 120.0])
                .metric("part", vec![40.0, 44.0, 48.0])
                .counter("commands", 7),
        ])
}

/// The hydration suite labels its runs with the git branch, and branches have slashes in them.
#[test]
fn a_label_with_a_slash_in_it_still_names_one_file() {
    let labelled = |label: &str| {
        Run::new(&SUITE, Meta::default(), label.to_string())
            .file_label()
            .to_string()
    };

    assert_eq!(labelled("feat/hydration-rust"), "feat-hydration-rust");
    // Everything a well-behaved label is made of survives untouched.
    assert_eq!(labelled("abc1234-dirty"), "abc1234-dirty");
    assert_eq!(labelled("js_hydration.2"), "js_hydration.2");
    // Whatever is left has to be a filename, not an empty string or a directory traversal.
    assert_eq!(labelled(""), "run");
    assert_eq!(labelled(".."), "run");
    // Dots survive, but nothing that would let the name be read as more than one component -
    // so this is a silly filename rather than a path out of `target/bench/`.
    assert_eq!(labelled("../../etc/passwd"), "..-..-etc-passwd");
}

/// The guard's job is to warn; the one failure it must not have is resolving to nothing.
#[test]
fn a_guard_pointer_escapes_what_rfc_6901_reserves() {
    assert_eq!(
        json_pointer(&["machine", "cpu_model"]),
        "/machine/cpu_model"
    );
    // A harness key is whatever a suite passed to `Run::harness`, so it may hold either of the
    // two characters a pointer gives a meaning to.
    assert_eq!(
        json_pointer(&["harness", "scale.factor"]),
        "/harness/scale.factor"
    );
    assert_eq!(json_pointer(&["harness", "a/b"]), "/harness/a~1b");
    assert_eq!(json_pointer(&["harness", "a~b"]), "/harness/a~0b");
}

/// A run whose harness key holds a `.` has to be found in the JSON by the same pointer the
/// baseline stored, or the field is dropped and the comparison guards nothing.
#[test]
fn a_harness_key_holding_a_dot_is_still_found_in_a_run() {
    let run = sample_run().harness("scale.factor", 0.5);
    let json = run.to_json(1_700_000_000);

    assert_eq!(
        json.pointer(&json_pointer(&["harness", "scale.factor"])),
        Some(&serde_json::json!(0.5))
    );
}

#[test]
fn a_run_round_trips_through_its_own_file_format() {
    let written = sample_run().to_json(1_700_000_000);
    let path = write_temp("round-trip.json", &written);

    let baseline = match Baseline::load(&path, &SUITE) {
        Ok(baseline) => baseline,
        Err(err) => panic!("loading what we just wrote failed: {err}"),
    };

    assert_eq!(baseline.label, "under-test");
    assert_eq!(baseline.schema, crate::SCHEMA);
    assert_eq!(baseline.metric("app /one", "total").median, 110.0);
    assert_eq!(baseline.counter("app /one", "commands"), Some(7));
}

#[test]
fn a_baseline_from_another_suite_is_refused() {
    let mut written = sample_run().to_json(1_700_000_000);
    written["kind"] = serde_json::json!("vertigo-some-other-bench");
    let path = write_temp("wrong-kind.json", &written);

    let Err(err) = Baseline::load(&path, &SUITE) else {
        panic!("a run of a different suite should not load");
    };

    assert!(
        err.to_string().contains("vertigo-some-other-bench"),
        "the error should name what it found: {err}"
    );
}

/// The shape `tests/ssr-bench` wrote before the four suites were unified.
///
/// Worth a test rather than a comment: the runs that are expensive to reproduce are exactly the
/// old ones - a hydration baseline from another branch costs a worktree and a cherry-pick - so
/// this is the path that keeps them usable.
#[test]
fn a_schema_1_ssr_baseline_still_reads() {
    let old = serde_json::json!({
        "schema": 1,
        "kind": "vertigo-test-bench",
        "created_at_unix": 1_600_000_000_u64,
        "label": "abc1234",
        "machine": { "cpu_model": "a cpu", "governor": "performance" },
        "toolchain": { "rustc": "rustc 1.90.0" },
        "harness": { "warmup": 3, "samples": 15 },
        "rows": [{
            "subject": "app",
            "route": "/one",
            "status": 200,
            "total_us": { "min": 90.0, "median": 95.0, "p90": 99.0 },
            "phases_us": { "part": { "min": 30.0, "median": 35.0, "p90": 39.0 } },
            "counters": { "commands": 7 },
        }],
    });
    let path = write_temp("schema1-ssr.json", &old);

    let baseline = match Baseline::load(&path, &SUITE) {
        Ok(baseline) => baseline,
        Err(err) => panic!("an old SSR baseline should still load: {err}"),
    };

    assert_eq!(baseline.schema, 1);
    // `total_us` sat beside the phases rather than among them; it is a metric now.
    assert_eq!(baseline.metric("app /one", "total").median, 95.0);
    assert_eq!(baseline.metric("app /one", "part").median, 35.0);
    // `status` was a field of its own; the counters were one object among several.
    assert_eq!(baseline.counter("app /one", "status"), Some(200));
    assert_eq!(baseline.counter("app /one", "commands"), Some(7));
}

/// The hydration shape, whose key carried a third component and whose counters were spread
/// across two objects - one of which could be `null`.
#[test]
fn a_schema_1_hydration_baseline_still_reads() {
    let old = serde_json::json!({
        "schema": 1,
        "kind": "vertigo-test-bench",
        "created_at_unix": 1_600_000_000_u64,
        "label": "js-hydration",
        "rows": [
            {
                "subject": "bench",
                "route": "/wide",
                "hydration": "on",
                "phases_ms": { "total": { "min": 31.6, "median": 33.7, "p90": 37.2 } },
                "mutations": { "mutations": 9045, "added": 3010 },
                "coverage": { "rootFound": true, "matched": 3007, "hydratable": 3007 },
            },
            {
                "subject": "bench",
                "route": "/wide",
                "hydration": "off",
                "phases_ms": { "total": { "min": 3.1, "median": 3.3, "p90": 3.6 } },
                "mutations": { "mutations": 27, "added": 7 },
                "coverage": serde_json::Value::Null,
            },
        ],
    });
    let path = write_temp("schema1-hydration.json", &old);

    let baseline = match Baseline::load(&path, &SUITE) {
        Ok(baseline) => baseline,
        Err(err) => panic!("an old hydration baseline should still load: {err}"),
    };

    assert_eq!(baseline.metric("bench /wide [on]", "total").median, 33.7);
    assert_eq!(
        baseline.counter("bench /wide [on]", "mutations"),
        Some(9045)
    );
    assert_eq!(baseline.counter("bench /wide [on]", "matched"), Some(3007));
    // `rootFound` was a boolean, and is worth noticing when it changes.
    assert_eq!(baseline.counter("bench /wide [on]", "rootFound"), Some(1));

    // A null coverage block leaves the keys absent, which is what the `[off]` rows are checked
    // on - absent is a different fact from zero.
    assert_eq!(baseline.counter("bench /wide [off]", "matched"), None);
    assert_eq!(baseline.counter("bench /wide [off]", "mutations"), Some(27));
}

#[test]
fn a_schema_from_the_future_is_refused() {
    let mut written = sample_run().to_json(1_700_000_000);
    written["schema"] = serde_json::json!(999);
    let path = write_temp("schema999.json", &written);

    assert!(Baseline::load(&path, &SUITE).is_err());
}

#[test]
fn a_metric_with_no_observations_is_absent_rather_than_zero() {
    let row = Row::new("demo /one").metric("total", vec![]);

    assert!(!row.metrics.contains_key("total"));
    // The table prints `-` for it. A zero would read as "instant" rather than "not measured".
    assert_eq!(row.get_metric("total"), Stats::default());
}

#[test]
fn counters_a_row_does_not_carry_are_absent_rather_than_zero() {
    let row = Row::new("app /one").counter("commands", 0);

    assert_eq!(row.get_counter("commands"), 0);
    assert!(row.counters.contains_key("commands"));
    assert!(!row.counters.contains_key("mutations"));
    // The getter cannot tell the two apart, which is why the callers that care read the map.
    assert_eq!(row.get_counter("mutations"), 0);
}

#[test]
fn metrics_and_counters_survive_into_the_json_under_their_own_names() {
    let json = sample_run().to_json(1_700_000_000);

    assert_eq!(
        json.pointer("/rows/0/key"),
        Some(&serde_json::json!("app /one"))
    );
    assert_eq!(
        json.pointer("/rows/0/metrics/total/median"),
        Some(&serde_json::json!(110.0))
    );
    assert_eq!(
        json.pointer("/rows/0/counters/commands"),
        Some(&serde_json::json!(7))
    );
    // Declared once at the top so a reader knows the intended order and units without knowing
    // which suite wrote the file.
    assert_eq!(
        json.pointer("/metrics/0/name"),
        Some(&serde_json::json!("total"))
    );
    assert_eq!(json.pointer("/headline"), Some(&serde_json::json!("total")));
}

// --------------------------------------------------------------------------------------
// artifacts
// --------------------------------------------------------------------------------------

/// A wasm module with two sections whose sizes are known by construction.
///
/// Section 10 is `code`, section 11 is `data`. Each is `id`, then a one-byte LEB128 length,
/// then that many bytes of payload - so the payload sizes below are exactly what the parser
/// should report, and the file is four bytes larger than their sum plus the eight-byte header.
fn minimal_wasm() -> Vec<u8> {
    let mut out = b"\0asm\x01\x00\x00\x00".to_vec();

    out.push(10);
    out.push(5);
    out.extend([0u8; 5]);

    out.push(11);
    out.push(3);
    out.extend([0u8; 3]);

    out
}

#[test]
fn wasm_sections_are_read_by_payload_size() {
    let artifacts = scan_temp_build(
        "app",
        "wasm-sections-are-read-by-payload-size",
        &minimal_wasm(),
        b"// javascript",
    );
    let wasm = find_artifact(&artifacts, "app wasm");

    assert_eq!(wasm.parts.get("code"), Some(&5));
    assert_eq!(wasm.parts.get("data"), Some(&3));
    // The header and the two id/length pairs belong to no section, which is the same
    // convention `wasm-objdump -h` prints.
    assert_eq!(wasm.bytes, 8 + 2 + 5 + 2 + 3);
}

#[test]
fn anything_that_is_not_wasm_has_no_sections() {
    let artifacts = scan_temp_build(
        "app",
        "anything-that-is-not-wasm-has-no-sections",
        &minimal_wasm(),
        b"// javascript",
    );
    let js = find_artifact(&artifacts, "app js");

    assert!(js.parts.is_empty());
    assert_eq!(js.bytes, 13);
}

#[test]
fn a_truncated_module_reports_what_it_could_read() {
    // The `data` section claims more bytes than the file holds.
    let mut broken = b"\0asm\x01\x00\x00\x00".to_vec();
    broken.push(10);
    broken.push(2);
    broken.extend([0u8; 2]);
    broken.push(11);
    broken.push(200);
    broken.extend([0u8; 4]);

    let artifacts = scan_temp_build(
        "broken",
        "a-truncated-module-reports-what-it-could-read",
        &broken,
        b"x",
    );
    let wasm = find_artifact(&artifacts, "broken wasm");

    assert_eq!(wasm.parts.get("code"), Some(&2));
    assert_eq!(wasm.parts.get("data"), None);
}

#[test]
fn scan_strips_the_public_path_placeholder_and_gzips() {
    let artifacts = scan_temp_build(
        "app",
        "scan-strips-the-public-path-placeholder-and-gzips",
        &minimal_wasm(),
        b"// javascript",
    );

    assert_eq!(artifacts.len(), 2);
    // The name in `index.json` carries a `%%...%%/` prefix the server substitutes; the file is
    // in the dest dir under the basename.
    assert!(
        find_artifact(&artifacts, "app wasm")
            .file
            .ends_with(".wasm")
    );
    assert!(!find_artifact(&artifacts, "app wasm").file.contains('/'));
    // Compressing anything at all produces a gzip member, so this is never zero.
    assert!(find_artifact(&artifacts, "app js").gzip_bytes > 0);
}

#[test]
fn a_missing_build_is_an_error_rather_than_zero_bytes() {
    let missing = std::env::temp_dir().join("vertigo-bench-report-tests/not-a-build");
    let _ = std::fs::remove_dir_all(&missing);

    assert!(Artifact::scan("app", &missing.to_string_lossy()).is_err());
}

#[test]
fn a_byte_delta_is_exact_and_signed() {
    assert_eq!(delta_bytes(999_328, 1_042_153), "+42 825 B (+4.3%)");
    assert_eq!(delta_bytes(1_042_153, 999_328), "-42 825 B (-4.1%)");
    // Unlike the timings there is no noise band: one byte is one byte.
    assert_eq!(delta_bytes(1_000, 1_001), "+1 B (+0.1%)");
    assert_eq!(delta_bytes(1_000, 1_000), "-");
    assert_eq!(delta_bytes(0, 512), "+512 B");
}

#[test]
fn digits_are_grouped_in_threes() {
    assert_eq!(bytes(0), "0");
    assert_eq!(bytes(999), "999");
    assert_eq!(bytes(1_000), "1 000");
    assert_eq!(bytes(1_042_153), "1 042 153");
}

#[test]
fn a_rebuild_with_the_same_size_is_still_a_change() {
    let before = Artifact {
        name: "app wasm".to_string(),
        file: "app.aaaa.wasm".to_string(),
        bytes: 1_000,
        ..Artifact::default()
    };
    let after = Artifact {
        file: "app.bbbb.wasm".to_string(),
        ..before.clone()
    };

    assert_eq!(before.bytes, after.bytes);
    // The hashed filename is content-addressed, which is what makes this detectable at all.
    assert!(!before.same_content(&after));
}

#[test]
fn artifacts_round_trip_through_the_file_format() {
    let run = sample_run().artifacts(scan_temp_build(
        "app",
        "artifacts-round-trip-through-the-file-format",
        &minimal_wasm(),
        b"// javascript",
    ));
    let path = write_temp("artifacts.json", &run.to_json(1_700_000_000));

    let baseline = match Baseline::load(&path, &SUITE) {
        Ok(baseline) => baseline,
        Err(err) => panic!("loading what we just wrote failed: {err}"),
    };

    let wasm = find_artifact(baseline.artifacts_for_test(), "app wasm");
    assert_eq!(wasm.bytes, 8 + 2 + 5 + 2 + 3);
    assert_eq!(wasm.parts.get("code"), Some(&5));
}

#[test]
fn a_schema_1_baseline_carries_no_artifacts() {
    let old = serde_json::json!({
        "schema": 1,
        "kind": "vertigo-test-bench",
        "label": "abc1234",
        "rows": [],
    });
    let path = write_temp("schema1-no-artifacts.json", &old);

    let baseline = match Baseline::load(&path, &SUITE) {
        Ok(baseline) => baseline,
        Err(err) => panic!("an old baseline should still load: {err}"),
    };

    // Empty rather than "everything is new", which is what the comparison says out loud.
    assert!(baseline.artifacts_for_test().is_empty());
}

fn find_artifact<'a>(artifacts: &'a [Artifact], name: &str) -> &'a Artifact {
    match artifacts.iter().find(|artifact| artifact.name == name) {
        Some(artifact) => artifact,
        None => panic!("no artifact named {name}"),
    }
}

/// A dest dir shaped the way `vertigo build` leaves one, scanned.
///
/// `slot` names the directory and `label` names the artifacts, which is why they are two
/// arguments rather than one. Cargo runs these tests concurrently, and `fs::write` truncates
/// before it writes - so two tests sharing a directory can have one of them scan a file the
/// other has just emptied, and fail in a way that reads as a broken LEB128 parser. Every
/// caller passes its own name.
fn scan_temp_build(label: &str, slot: &str, wasm: &[u8], js: &[u8]) -> Vec<Artifact> {
    let dir = std::env::temp_dir().join(format!("vertigo-bench-report-tests/build-{slot}"));
    let _ = std::fs::create_dir_all(&dir);

    let write = |name: &str, body: &[u8]| {
        if let Err(err) = std::fs::write(dir.join(name), body) {
            panic!("writing {name}: {err}");
        }
    };

    write("app.aaaa.wasm", wasm);
    write("wasm_run.bbbb.js", js);
    write(
        "index.json",
        serde_json::json!({
            "run_js": "%%VERTIGO_PUBLIC_BUILD_PATH%%/wasm_run.bbbb.js",
            "wasm": "%%VERTIGO_PUBLIC_BUILD_PATH%%/app.aaaa.wasm",
        })
        .to_string()
        .as_bytes(),
    );

    match Artifact::scan(label, &dir.to_string_lossy()) {
        Ok(artifacts) => artifacts,
        Err(err) => panic!("scanning the fixture build failed: {err}"),
    }
}

fn write_temp(name: &str, value: &serde_json::Value) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("vertigo-bench-report-tests");
    let _ = std::fs::create_dir_all(&dir);

    let path = dir.join(name);
    let body = match serde_json::to_string_pretty(value) {
        Ok(body) => body,
        Err(err) => panic!("serialising the fixture failed: {err}"),
    };

    if let Err(err) = std::fs::write(&path, body) {
        panic!("writing {}: {err}", path.display());
    }

    path
}

/// Read-only accessors the tests need and nothing else does.
impl Baseline {
    fn metric(&self, key: &str, name: &str) -> Stats {
        self.row_metrics(key)
            .and_then(|metrics| metrics.get(name).copied())
            .unwrap_or_default()
    }

    fn counter(&self, key: &str, name: &str) -> Option<i64> {
        self.row_counters(key)?.get(name).copied()
    }
}
