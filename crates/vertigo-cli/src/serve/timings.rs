//! Per-request SSR phase timings.
//!
//! Serving a page is one wall-clock number in the log, which cannot say *where* a
//! regression landed. This splits a render into the phases the work actually divides into,
//! for the benchmark in `tests/ssr-bench`.
//!
//! ## The phases, and why two of them overlap
//!
//! 1. **instantiation** - [`SsrTimings::instantiate`]. A fresh `Store` and `Instance` per
//!    request.
//! 2. **command generation in wasm** - [`SsrTimings::wasm_self`]. Time inside the wasm
//!    calls, *minus* the host work wasm called back out to do.
//! 3. **host DOM tree from those commands** - [`SsrTimings::dom_build`].
//! 4. **that tree to an HTML string** - [`SsrTimings::html_total`].
//!
//! Phases 2 and 3 are not two halves of a stopwatch. The command blob is decoded on the
//! host inside `import_dom_access`, while the wasm call that produced it is still on the
//! stack - so `decode_dom` is phase-3 work measured from inside a phase-2 region. Both raw
//! numbers are kept and the accessors do the arithmetic, so a reader can check it rather
//! than trust it.
//!
//! ## The probe
//!
//! [`SsrProbe`] exists in both builds; only with the `ssr-timings` feature does it record
//! anything. That is what keeps `#[cfg]` out of the four files that call it - they read
//! identically either way, and `Instant` is imported in exactly one place.
//!
//! Timing is [`SsrProbe::start`] to get a [`Mark`], then a named method to spend it.
//! Deliberately not an RAII guard (it would record on the error-path `return`s in
//! `html_build_response`, invisibly) and deliberately not a `measure(|| ...)` closure - at
//! two of the call sites the closure would capture `&mut self` while `self.probe` is
//! already borrowed, and would not compile.

/// A clock reading taken by [`SsrProbe::start`], spent by exactly one record call.
///
/// Carries no field without the feature, so it is a zero-sized value that the call sites
/// pass around for free. Defined here rather than inside the two modules below so that the
/// name is the same type in both builds and needs no re-export.
#[derive(Clone, Copy)]
pub struct Mark {
    #[cfg(feature = "ssr-timings")]
    at: std::time::Instant,
}

impl Mark {
    #[cfg(feature = "ssr-timings")]
    fn now() -> Self {
        Self {
            at: std::time::Instant::now(),
        }
    }

    #[cfg(not(feature = "ssr-timings"))]
    #[inline(always)]
    fn now() -> Self {
        Self {}
    }

    #[cfg(feature = "ssr-timings")]
    fn elapsed(self) -> std::time::Duration {
        self.at.elapsed()
    }
}

#[cfg(feature = "ssr-timings")]
mod on {
    use parking_lot::Mutex;
    use std::{sync::Arc, time::Duration};

    use super::Mark;

    /// One request's breakdown. All durations are cumulative over the request.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct SsrTimings {
        /// The whole of `ServerState::request_inner`. Will not equal the sum of the parts -
        /// see [`SsrTimings::unaccounted`].
        pub total: Duration,

        // -- phase 1 ------------------------------------------------------------------
        /// `WasmInstance::new`: `Store::new` and `InstancePre::instantiate`. Imports were
        /// resolved by name once at startup, so none of that cost lands here.
        pub instantiate: Duration,

        // -- phase 2, and the host work nested inside it --------------------------------
        /// Wall time of every call into wasm, summed. Host callbacks are *inside* this.
        pub wasm_wall: Duration,
        /// Of `wasm_wall`: `vertigo_entry_function`, i.e. the app mounting.
        pub wasm_mount: Duration,
        /// Of `wasm_wall`: `vertigo_export_handle_url`.
        pub wasm_handle_url: Duration,
        /// Of `wasm_wall`: `vertigo_export_wasm_command` - timer callbacks and fetch
        /// responses re-entering wasm from the drain loop.
        pub wasm_reentry: Duration,
        /// Host time inside `import_dom_access`, summed: reading the argument out of linear
        /// memory, dispatching it, writing the answer back. Nested inside `wasm_wall`.
        pub host_in_wasm: Duration,
        /// Of `host_in_wasm`: `decode_dom_commands` alone.
        pub decode_dom: Duration,

        // -- phase 3 --------------------------------------------------------------------
        /// `AllElements::feed`, summed over every batch. Runs in the drain loop, outside
        /// any wasm call.
        pub dom_apply: Duration,

        // -- phase 4 --------------------------------------------------------------------
        /// `HtmlResponse::build_response` end to end: the three below plus slack.
        pub build_response: Duration,
        /// `AllElements::get_response` - the `HtmlNode` tree and the `<style>`.
        pub html_tree: Duration,
        /// head/body injection, including serialising the SSR fetch cache.
        pub html_inject: Duration,
        /// `convert_to_string` plus the two placeholder substitutions.
        pub html_string: Duration,

        // -- waiting, not working -------------------------------------------------------
        /// Time the drain loop spent parked waiting for an SSR fetch. Named so it cannot
        /// hide inside `total` and be read as work.
        pub fetch_wait: Duration,

        // -- counters -------------------------------------------------------------------
        /// `DomBulkUpdate` batches received.
        pub dom_batches: u32,
        /// `DriverDomCommand`s decoded across all batches.
        pub dom_commands: u32,
        /// Encoded wire bytes across all batches.
        pub dom_blob_bytes: u64,
        /// `import_dom_access` crossings in total - DOM batches and everything else.
        pub host_calls: u32,
        /// Calls into wasm: one mount, one `handle_url`, one per re-entry.
        pub wasm_calls: u32,
        /// Of those, `vertigo_export_wasm_command`.
        pub reentry_calls: u32,
        /// Outbound fetches actually issued (cache misses that spawned a request).
        pub fetches: u32,
        /// Bytes of the final response body.
        pub html_bytes: u64,
    }

    impl SsrTimings {
        /// Phase 2: wasm's own execution, with the host work it called back out to do
        /// removed.
        ///
        /// `saturating_sub` throughout this impl: the inner and outer clocks are read at
        /// different nesting depths, so at microsecond scale an inner sum can exceed its
        /// outer by a tick. `Duration`'s `Sub` panics on underflow.
        pub fn wasm_self(&self) -> Duration {
            self.wasm_wall.saturating_sub(self.host_in_wasm)
        }

        /// Phase 3: the wire decode (inside the wasm call) plus applying the commands
        /// (outside it).
        pub fn dom_build(&self) -> Duration {
            self.decode_dom + self.dom_apply
        }

        /// Phase 4.
        pub fn html_total(&self) -> Duration {
            self.html_tree + self.html_inject + self.html_string
        }

        /// What `total` holds that no phase claimed: the channel, the timeout task, the
        /// per-call `get_typed_func` export lookups, allocator and scheduler noise.
        ///
        /// A benchmark should print this. Large and positive means a phase is missing a
        /// timer; negative - which the saturating arithmetic here renders as zero, so watch
        /// the per-sample figure instead - means something is being counted twice.
        pub fn unaccounted(&self) -> Duration {
            self.total
                .saturating_sub(self.instantiate)
                // `host_in_wasm`, and so `decode_dom`, is already inside `wasm_wall`.
                .saturating_sub(self.wasm_wall)
                .saturating_sub(self.dom_apply)
                .saturating_sub(self.build_response)
                .saturating_sub(self.fetch_wait)
        }
    }

    /// Shared, `Send + Sync`, cheap to clone.
    ///
    /// `Arc<Mutex<_>>` rather than `&mut`, because the accumulator has to be reachable from
    /// the `handle_command` closure, which is an `Arc<dyn Fn + Send + Sync>`, and from the
    /// `Func::wrap` import, which carries the same bound. `parking_lot` rather than `std`
    /// because its `lock` returns the guard directly - the workspace denies `unwrap_used`
    /// and `expect_used`, and a poisoned-lock `unwrap` at every record site would be the
    /// only reason either lint would fire here.
    ///
    /// **The lock is never held across measured code.** Every method reads the elapsed time
    /// first and takes the guard second. This is load-bearing rather than tidy:
    /// [`SsrProbe::decoded`] records from inside the region [`SsrProbe::host_call`] is
    /// timing, and `parking_lot::Mutex` is not reentrant.
    #[derive(Clone, Default)]
    pub struct SsrProbe(Arc<Mutex<SsrTimings>>);

    impl SsrProbe {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn start(&self) -> Mark {
            Mark::now()
        }

        fn add(&self, mark: Mark, pick: impl FnOnce(&mut SsrTimings) -> &mut Duration) {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            *pick(&mut guard) += elapsed;
        }

        pub fn instantiate(&self, mark: Mark) {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            guard.instantiate += elapsed;
        }

        /// Keyed on the exported function name, so the one call site in
        /// `WasmInstance::call_function` buckets all three entry points.
        pub fn wasm_call(&self, name: &'static str, mark: Mark) {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            guard.wasm_wall += elapsed;
            guard.wasm_calls += 1;

            match name {
                super::ENTRY_FUNCTION => guard.wasm_mount += elapsed,
                super::HANDLE_URL_FUNCTION => guard.wasm_handle_url += elapsed,
                _ => {
                    guard.wasm_reentry += elapsed;
                    guard.reentry_calls += 1;
                }
            }
        }

        pub fn host_call(&self, mark: Mark) {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            guard.host_in_wasm += elapsed;
            guard.host_calls += 1;
        }

        pub fn decoded(&self, mark: Mark, blob_bytes: usize, commands: usize) {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            guard.decode_dom += elapsed;
            guard.dom_batches += 1;
            guard.dom_commands += commands as u32;
            guard.dom_blob_bytes += blob_bytes as u64;
        }

        pub fn dom_apply(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.dom_apply);
        }

        pub fn build_response(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.build_response);
        }

        pub fn html_tree(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.html_tree);
        }

        pub fn html_inject(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.html_inject);
        }

        pub fn html_string(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.html_string);
        }

        pub fn fetch_wait(&self, mark: Mark) {
            self.add(mark, |timings| &mut timings.fetch_wait);
        }

        pub fn fetch_started(&self) {
            self.0.lock().fetches += 1;
        }

        /// Close the request: stamp `total` and `html_bytes`, and hand back the record.
        pub fn finish(&self, mark: Mark, body_len: usize) -> SsrTimings {
            let elapsed = mark.elapsed();
            let mut guard = self.0.lock();
            guard.total = elapsed;
            guard.html_bytes = body_len as u64;
            guard.clone()
        }
    }
}

#[cfg(not(feature = "ssr-timings"))]
mod off {
    use super::Mark;

    /// Zero-sized stand-in. Every method is an inlined no-op, so the call sites in
    /// `server_state`, `wasm_instance`, `html_response` and `html_build_response` compile
    /// away entirely.
    ///
    /// `Clone` but deliberately **not** `Copy`: the call sites clone the probe, which is
    /// what the feature-on `Arc` version requires, and `clippy::clone_on_copy` would
    /// otherwise fire on every one of them in exactly the build CI does not lint.
    #[derive(Clone, Default)]
    pub struct SsrProbe;

    impl SsrProbe {
        #[inline(always)]
        pub fn new() -> Self {
            Self
        }
        #[inline(always)]
        pub fn start(&self) -> Mark {
            Mark::now()
        }
        #[inline(always)]
        pub fn instantiate(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn wasm_call(&self, _name: &'static str, _mark: Mark) {}
        #[inline(always)]
        pub fn host_call(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn decoded(&self, _mark: Mark, _blob_bytes: usize, _commands: usize) {}
        #[inline(always)]
        pub fn dom_apply(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn build_response(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn html_tree(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn html_inject(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn html_string(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn fetch_wait(&self, _mark: Mark) {}
        #[inline(always)]
        pub fn fetch_started(&self) {}
    }
}

/// The wasm exports, named here so [`SsrProbe::wasm_call`] and the callers agree on the
/// spelling.
pub const ENTRY_FUNCTION: &str = "vertigo_entry_function";
pub const HANDLE_URL_FUNCTION: &str = "vertigo_export_handle_url";
pub const WASM_COMMAND_FUNCTION: &str = "vertigo_export_wasm_command";

#[cfg(feature = "ssr-timings")]
pub use on::{SsrProbe, SsrTimings};

#[cfg(not(feature = "ssr-timings"))]
pub use off::SsrProbe;
