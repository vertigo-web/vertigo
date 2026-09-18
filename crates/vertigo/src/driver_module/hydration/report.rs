use vertigo_macro::AutoJsJson;

use crate::{JsJsonSerialize, driver_module::api::DomAccess};

/// What hydration did with the batch it received.
///
/// Stored at `window.__vertigo_hydration` because that's how the browser test
/// (`tests/demo/ssr.rs`) reads it: wasm starts asynchronously, so there is no moment
/// when the test could inject a console.log mock and be certain it will catch the
/// printed line.
///
/// Field names are deliberately in camelCase - this is a contract with that test,
/// older than this version of hydration.
#[derive(AutoJsJson, Debug, Clone, Default)]
pub struct HydrationReport {
    /// False when the snapshot had no `<body>` - hydration had nowhere to start.
    #[js_json(rename = "rootFound")]
    pub root_found: bool,
    /// Number of adoptions emitted.
    pub matched: u64,
    /// Nodes that hydration had a chance to match, i.e. elements and texts under
    /// `<head>`/`<body>`. Excludes what cannot be matched - see `skipped`.
    pub hydratable: u64,
    /// Nodes without a counterpart in the server output: comment markers that the
    /// server cuts, and texts beyond the first in a merged run that the server
    /// glued into one pass. They don't count against the result because matching
    /// them is impossible, not failed.
    pub skipped: u64,
    /// Every identifier mentioned by the batch.
    pub total: u64,
}

impl HydrationReport {
    pub fn publish(&self) {
        DomAccess::default()
            .root("window")
            .set("__vertigo_hydration", self.clone().to_json())
            .exec();

        let percent = match self.hydratable {
            0 => 100.0,
            hydratable => self.matched as f64 * 100.0 / hydratable as f64,
        };

        let summary = format!(
            "Hydration complete: {}/{} matched ({percent:.2}%), {} skipped, {} nodes in batch.",
            self.matched, self.hydratable, self.skipped, self.total
        );

        if self.matched < self.hydratable {
            log::warn!("{summary}");
        } else {
            log::info!("{summary}");
        }
    }
}
