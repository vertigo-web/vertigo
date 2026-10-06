//! The check that catches a wasm panic.
//!
//! Vertigo's panic hook (`crates/vertigo/src/driver_module/init_env.rs`) routes a panic to
//! `api_panic_message().show(..)`, which the JS side prints as `console.error('PANIC', msg)`
//! (`src_js/wasm_module.ts`). Nothing about that reaches the DOM, so a panic in a click
//! handler is entirely invisible to element assertions - the app just quietly stops
//! responding, and only a later assertion notices, if one happens to cover that path.
//!
//! So: read what the console complained about, and require it to be nothing.
//!
//! Read from the browser's own log rather than from a recorder put into the page. The log
//! covers the page from its first byte, so a boot-time failure is caught for certain, and a
//! page load does not throw it away - nothing has to be put back after a `goto`. It also holds
//! what a recorder never sees, such as failed requests, which is why some of [`ALLOWED`] is
//! about those.

use vertigo_testing::prelude::*;

use crate::Ctx;

/// Patterns that are the environment misbehaving rather than the app.
///
/// Deliberately short, and every entry says why it is here. Nothing that originates in vertigo
/// or in the demo belongs on this list - if a demo action logs an error, that is the test
/// doing its job.
const ALLOWED: &[(&str, &str)] = &[
    // The clipboard tab calls `navigator.clipboard.writeText`. That needs both a permission
    // grant and a focused document; a browser run under a WebDriver reliably has neither.
    (
        "NotAllowedError",
        "clipboard write is not permitted under WebDriver",
    ),
    ("Document is not focused", "same, as Chrome words it"),
    // `window.scrollMaxY` is a Firefox extension, and the demo says so on the button itself
    // ("scroll to bottom (FF)"). Elsewhere it reads as undefined.
    (
        "scrollMaxY",
        "the button is labelled Firefox-only in the demo",
    ),
    // A page without an icon link - the plain-text robots.txt - makes Chrome ask for
    // `/favicon.ico`, which the demo does not have. The request is the browser's, not the app's.
    (
        "/favicon.ico - Failed to load resource",
        "Chrome looks for an icon the plain-text robots.txt cannot name",
    ),
];

/// Findings this test has already made, which are open rather than accepted.
///
/// Kept apart from [`ALLOWED`] on purpose. An allowlisted message is the environment being
/// itself and will always be there; one of these is a real defect that the run is tolerating
/// so that the other thirteen tabs can still be checked. Each is printed loudly on every run,
/// and deleting the entry is what closing the issue looks like.
///
/// Empty, and worth keeping empty. The last entry here was `keyed_computed_list` reporting a
/// read after removal when the List tab dropped a row; that is now covered by
/// `removing_a_row_does_not_report_a_read_after_removal` in the vertigo unit tests.
///
/// Kept rather than deleted now that it is empty. It is four lines, and it is the difference
/// between a tolerated defect being recorded with a reason and someone quietly widening
/// [`ALLOWED`] instead - which is where a real bug would go to be forgotten.
const KNOWN_ISSUES: &[(&str, &str)] = &[];

fn matches(patterns: &[(&str, &str)], message: &str) -> bool {
    patterns
        .iter()
        .any(|(pattern, _)| message.contains(pattern))
}

/// Lets the harness's own check, after the last step, apply the same allowlist.
pub fn allow_in_harness(ctx: &Ctx) {
    for (pattern, _) in ALLOWED.iter().chain(KNOWN_ISSUES) {
        ctx.allow_console(pattern);
    }
}

/// Drain the log and fail on any error left after the allowlist.
///
/// Drained rather than merely read, and called after every tab rather than once at the end, so
/// that a message names the tab that produced it.
pub async fn assert_clean(ctx: &Ctx, stage: &str) -> Result<()> {
    assert_clean_except(ctx, stage, &[]).await
}

/// [`assert_clean`] for a stage which provokes some errors on purpose: messages containing one
/// of `expected` are its own doing.
pub async fn assert_clean_except(ctx: &Ctx, stage: &str, expected: &[&str]) -> Result<()> {
    // Chrome hands each entry out once, so this reads what came since the previous stage
    let logs = ctx
        .driver
        .browser_log()
        .await
        .context("reading the browser log failed")?;

    let mut unexpected = Vec::new();
    let mut ignored = Vec::new();
    let mut known = Vec::new();

    for entry in logs.into_iter().filter(|entry| entry.level == "SEVERE") {
        let message = entry.message;

        if matches(KNOWN_ISSUES, &message) {
            known.push(message);
        } else if matches(ALLOWED, &message)
            || expected.iter().any(|pattern| message.contains(pattern))
        {
            ignored.push(message);
        } else {
            unexpected.push(message);
        }
    }

    if !ignored.is_empty() {
        println!(
            "     console ({stage}): {} allowlisted message(s) ignored:",
            ignored.len()
        );
        for message in &ignored {
            println!("       - {message}");
        }
    }

    for message in &known {
        println!("  !! KNOWN ISSUE during {stage}: {message}");
        for (pattern, note) in KNOWN_ISSUES {
            if message.contains(pattern) {
                println!("     {note}");
            }
        }
    }

    ensure!(
        unexpected.is_empty(),
        "the browser console reported {} problem(s) during {stage}:\n{}",
        unexpected.len(),
        unexpected
            .iter()
            .map(|message| format!("  - {message}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    Ok(())
}
