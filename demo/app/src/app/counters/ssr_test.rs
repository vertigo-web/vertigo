//! What the server draws and what the browser draws, disagreeing on purpose.
//!
//! `vertigo serve` renders this component on the server and sends the result as HTML; the wasm
//! then renders it again in the browser and hydration has to reconcile the two. The trees below
//! differ in every way one reasonably can - in depth, in order, in how many children they have,
//! in what those children say and in which of them carry an `href` - and what must be standing
//! once hydration is done is the browser's tree, with nothing of the server's left behind.
//!
//! **Every difference here is deliberate.** Making the two sides agree would read as tidying up
//! and would quietly delete the test: two identical trees say nothing about whether hydration
//! reconciles anything. Each line says what it is there to catch.

use vertigo::{component, dom, get_driver};

#[component]
pub fn SsrTest() {
    if get_driver().is_browser() {
        return dom! {
            <div>
                // Three levels deep where the server sent one. Hydration has to build the
                // missing depth rather than reuse the flat nodes it was given.
                <div>
                    <div>
                        <div>"Only the browser draws this, three levels down"</div>
                    </div>
                </div>

                // Two anchors where the server sent three, and none of them with an href:
                // hydration has to drop the extra one and take the attributes off these.
                <a>"Shared link one"</a>
                <a>"Shared link two"</a>

                <div>"Rendered by: browser"</div>

                // The same two values the server sent, in the other order. Hydration that
                // matched nodes by position alone would leave these the wrong way round.
                <input type="text" value="field two" />
                <input type="text" value="field one" />
            </div>
        };
    }

    dom! {
        <div>
            <div>"Rendered by: server"</div>

            // Neither of these exists in the browser's tree, so both have to go.
            <hr/>
            <a>"Only the server draws this link"</a>

            <input type="text" value="field one" />
            <input type="text" value="field two" />

            <a href="/server-href-one">"Shared link one"</a>
            <a href="/server-href-two">"Shared link two"</a>
        </div>
    }
}
