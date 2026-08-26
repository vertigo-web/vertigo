//! The check that catches a wasm panic.
//!
//! Vertigo's panic hook (`crates/vertigo/src/driver_module/init_env.rs`) routes a panic to
//! `api_panic_message().show(..)`, which the JS side prints as `console.error('PANIC', msg)`
//! (`src_js/wasm_module.ts`). Nothing about that reaches the DOM, so a panic in a click
//! handler is entirely invisible to element assertions - the app just quietly stops
//! responding, and only a later assertion notices, if one happens to cover that path.
//!
//! So: record everything the console complains about, and require the tally to be empty.

use fantoccini::Client;
use serde_json::Value;

/// Installs the recorder. Anything the page reports from here on is collected into
/// `window.__vertigoErrors`.
///
/// **Known gap:** this runs after `goto` resolves, which is after the document has loaded but
/// possibly before the wasm has finished booting - so it usually, but not certainly, catches a
/// boot-time failure. The landmark assertions cover that case from the other side: a demo that
/// failed to boot renders nothing to find.
pub async fn install(client: &Client) {
    client
        .execute(
            r#"
            window.__vertigoErrors = [];
            const record = (kind, text) => window.__vertigoErrors.push(kind + ': ' + text);

            const original = console.error.bind(console);
            console.error = (...args) => {
                record('console.error', args.map((a) => {
                    try { return typeof a === 'string' ? a : JSON.stringify(a); }
                    catch (_) { return String(a); }
                }).join(' '));
                original(...args);
            };

            window.addEventListener('error', (event) => {
                record('window.onerror', String(event.message));
            });

            window.addEventListener('unhandledrejection', (event) => {
                record('unhandledrejection', String(event.reason));
            });
            "#,
            vec![],
        )
        .await
        .expect("installing the console recorder failed");
}

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
];

/// Findings this test has already made, which are open rather than accepted.
///
/// Kept apart from [`ALLOWED`] on purpose. An allowlisted message is the environment being
/// itself and will always be there; one of these is a real defect that the run is tolerating
/// so that the other thirteen tabs can still be checked. Each is printed loudly on every run,
/// and deleting the entry is what closing the issue looks like.
const KNOWN_ISSUES: &[(&str, &str)] = &[(
    "was read after that key left the source list",
    "Removing a row from the List tab reaches `keyed_computed_list`'s read-after-removal \
     fallback. The rendered output is right - the fallback returns the last value - but the \
     row's `Computed` is being refreshed after its key has gone, which the framework logs at \
     error level because it is not supposed to happen. \
     Reproduces natively, and needs three things together: the source is a \
     `Computed<Vec<Computed<T>>>` keyed on `Computed::id()`, a second subscriber is live on \
     the same source, and a row is removed. With the keyed list as the only subscriber it \
     does not fire, which points at the order pending updates are flushed in rather than at \
     the list reconciler.",
)];

fn matches(patterns: &[(&str, &str)], message: &str) -> bool {
    patterns
        .iter()
        .any(|(pattern, _)| message.contains(pattern))
}

fn is_allowed(message: &str) -> bool {
    matches(ALLOWED, message) || matches(KNOWN_ISSUES, message)
}

/// Drain the tally and fail on anything left after the allowlist.
///
/// Drained rather than merely read, and called after every tab rather than once at the end, so
/// that a message names the tab that produced it. A single check at the end would say only
/// that something, somewhere, went wrong.
pub async fn assert_clean(client: &Client, stage: &str) {
    let recorded = client
        .execute(
            "const found = window.__vertigoErrors || []; window.__vertigoErrors = []; return found;",
            vec![],
        )
        .await
        .expect("reading the console recorder failed");

    let Value::Array(entries) = recorded else {
        panic!("the console recorder returned {recorded:?} rather than an array");
    };

    let mut unexpected = Vec::new();
    let mut ignored = Vec::new();
    let mut known = Vec::new();

    for entry in entries {
        let message = match entry {
            Value::String(message) => message,
            other => other.to_string(),
        };

        if matches(KNOWN_ISSUES, &message) {
            known.push(message);
        } else if is_allowed(&message) {
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

    assert!(
        unexpected.is_empty(),
        "the browser console reported {} problem(s) during {stage}:\n{}",
        unexpected.len(),
        unexpected
            .iter()
            .map(|message| format!("  - {message}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}
