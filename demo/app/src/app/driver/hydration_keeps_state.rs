//! What a visitor does with the page before the app has started has to survive the start.
//!
//! Hydration adopts the nodes the server rendered instead of building new ones, and that is what
//! keeps the state the browser gave them: the field a visitor clicked into stays focused, and the
//! frame stays loaded. A node taken out of the document and put back, even right where it was,
//! loses both - and an iframe's `srcdoc` set again, even to the value it has, reloads the frame.
//!
//! Unlike [`SsrTest`](super::ssr_test::SsrTest), the server and the browser draw the same tree
//! here, so every node has one to be adopted by. And the field and the frame are kept on one
//! line - the frame as an inline block, which the Tailwind preflight of the demo makes a block
//! otherwise - where whitespace between them in the server's HTML would show as a gap: one the
//! browser's tree doesn't have, so the frame would move when the app starts.

use vertigo::{component, css, dom};

/// The frame counts its loads on the page, and shows the count.
const FRAME: &str = "<p>Loaded <b id='loads'></b> time(s) since the page was.</p>\
    <script>document.getElementById('loads').textContent = parent.frameLoads = (parent.frameLoads || 0) + 1</script>";

#[component]
pub fn HydrationKeepsState() {
    dom! {
        <div css={css! {"white-space: nowrap;"}}>
            <textarea placeholder="Click here before the app starts" />
            <iframe css={css! {"display: inline-block;"}} srcdoc={FRAME} />
        </div>
    }
}
