//! The in-page timing probe.
//!
//! An inline `<script>` the app renders into `<head>`. It is server-rendered, so it runs
//! while the browser is still parsing the document - long before wasm - and it is the only
//! part of this benchmark that is the same code on both implementations by construction,
//! because it is not part of vertigo at all.
//!
//! SSR emits `<script>` text unescaped (`html_node_convert_to_string.rs`, the `["script",
//! "style"]` arm), so the source below reaches the browser verbatim.
//!
//! ## Why the observer waits for `DOMContentLoaded`
//!
//! The HTML parser's own insertions are mutations, and a `MutationObserver` installed during
//! head parsing would record every node in the server-rendered body. That would swamp the
//! number this benchmark exists to report. Wasm boots from the `load` event, which always
//! fires after `DOMContentLoaded`, so arming the observer there excludes the parser and
//! still cannot miss anything vertigo does.
//!
//! ## Why it is idempotent
//!
//! If hydration fails to match the `<script>` element, the node is removed and re-created -
//! and a script element inserted through the DOM API executes again. The `window.__hb` guard
//! keeps the second run from resetting marks the first run already took.

use crate::SENTINEL_ID;

/// Where [`SENTINEL_ID`] is substituted into [`PROBE_TEMPLATE`].
const SENTINEL_PLACEHOLDER: &str = "__SENTINEL_ID__";

/// The probe, with the sentinel's id filled in.
pub fn probe_js() -> String {
    PROBE_TEMPLATE.replace(SENTINEL_PLACEHOLDER, SENTINEL_ID)
}

/// Recorded on `window.__hb`, read back by the driver after the page has settled.
const PROBE_TEMPLATE: &str = r#"
(function () {
  if (window.__hb) { return; }

  var hb = {
    t_probe: performance.now(),
    t_armed: null,
    t_hydrated: null,
    mutations: 0,
    added: 0,
    removed: 0,
    attrs: 0,
    chars: 0,
    done: false
  };
  window.__hb = hb;

  var observer = new MutationObserver(function (records) {
    for (var i = 0; i < records.length; i++) {
      var record = records[i];
      hb.mutations++;
      if (record.type === 'childList') {
        hb.added += record.addedNodes.length;
        hb.removed += record.removedNodes.length;
      } else if (record.type === 'attributes') {
        hb.attrs++;
      } else if (record.type === 'characterData') {
        hb.chars++;
      }
    }

    // The sentinel is in the server's markup too, carrying zeros, so its presence proves
    // nothing - a non-zero `data-end` is what says the application has been through it.
    // Records arrive as one microtask after a synchronous apply, so this fires at the end
    // of the apply rather than at some point inside it.
    if (!hb.done) {
      var sentinel = document.getElementById('__SENTINEL_ID__');
      if (sentinel && parseFloat(sentinel.getAttribute('data-end')) > 0) {
        hb.done = true;
        hb.t_hydrated = performance.now();
        observer.disconnect();
      }
    }
  });

  var arm = function () {
    hb.t_armed = performance.now();
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      characterData: true
    });
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', arm);
  } else {
    arm();
  }
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// The failure this guards against is silent and expensive: a probe looking for an id the
    /// app does not render never flips `hb.done`, so every page load in the suite fails on the
    /// sixty-second timeout instead of failing to compile.
    #[test]
    fn the_probe_looks_for_the_id_the_app_renders() {
        let script = probe_js();

        assert!(
            script.contains(&format!("getElementById('{SENTINEL_ID}')")),
            "the probe should look the sentinel up by SENTINEL_ID: {script}"
        );
        assert!(
            !script.contains(SENTINEL_PLACEHOLDER),
            "every placeholder should have been substituted"
        );
    }
}
