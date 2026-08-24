//! Browser clock and environment, reached through the `js!` macro.

use vertigo::{JsJson, js};

/// Wall clock in milliseconds, with sub-millisecond resolution.
///
/// [`vertigo::Instant`] is `Date.now()` - whole milliseconds - which cannot resolve the cost
/// of one graph operation. `performance.now()` can, but every call here is a wasm->JS round
/// trip, so it is only ever used twice per measured batch.
///
/// Browsers coarsen `performance.now()` (Chrome to ~100us outside a cross-origin-isolated
/// context). Batches are sized to 100-300ms, which keeps that below 0.1% - the reason
/// workloads are timed as a batch rather than per operation.
///
/// Returns 0.0 off-browser: during SSR the driver answers a JS call with `Null`. Harmless,
/// because the workloads never run there - see the `is_browser` gate in `lib.rs`.
pub fn now_ms() -> f64 {
    match js! { window.performance.now() } {
        JsJson::Number(number) => number.as_f64(),
        _ => 0.0,
    }
}

/// `?scale=0.25` shortens a run on a slow machine. Per-operation figures stay comparable,
/// since every workload divides by its own iteration count.
pub fn read_scale() -> f64 {
    // Parenthesised: a braced macro call cannot be the scrutinee of a `let ... else`.
    let JsJson::String(search) = js!(window.location.search) else {
        return 1.0;
    };

    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|part| part.strip_prefix("scale="))
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or(1.0)
}

/// Recorded next to the results, so a pasted table says which browser produced it.
pub fn user_agent() -> String {
    match js! { window.navigator.userAgent } {
        JsJson::String(text) => text,
        _ => "unknown".to_string(),
    }
}
