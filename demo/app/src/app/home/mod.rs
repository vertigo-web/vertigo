//! The landing page: what vertigo is, and what each of the other tabs is showing.
//!
//! Built by walking [`Route::ALL`], so the index cannot drift away from the menu: a tab added
//! to one appears in the other, with whatever `Route::about` says about it.

use vertigo::{Css, component, css, dom, dom_element};

use crate::app::route::Route;

#[component]
pub fn Home() {
    let entries = dom_element! {
        <div css={css_list()} />
    };

    for route in Route::ALL {
        // The index does not list itself, and `NotFound` is not in `ALL` to begin with.
        if *route == Route::Home {
            continue;
        }

        entries.add_child(dom! {
            <div css={css_entry()}>
                <a css={css_link()} href={route.to_string()}>{ route.label() }</a>
                <span css={css_about()}>{ route.about() }</span>
            </div>
        });
    }

    dom! {
        <div css={css_wrapper()}>
            <h1 css={css_title()}>"Vertigo"</h1>
            <p css={css_lead()}>
                "A reactive Real-DOM library with SSR for Rust. The graph of values and
                 computeds works out what to refresh when one of them changes, and the result
                 is written straight to the DOM - there is no virtual DOM in between."
            </p>
            <p css={css_lead()}>
                "The same WebAssembly renders this page on the server and in the browser, so
                 what you are reading arrived as HTML and was then taken over by the wasm."
            </p>
            <p css={css_lead()}>
                "Each tab below is a small demonstration of one part of that. Follow a link, or
                 step through them with the left and right arrow keys."
            </p>
            { entries }
            <p css={css_lead()}>
                "Source: "
                <a href="https://github.com/vertigo-web/vertigo" target="_blank">
                    "github.com/vertigo-web/vertigo"
                </a>
                " - the code for every tab is under "
                <code>"demo/app/src/app"</code>
                "."
            </p>
        </div>
    }
}

fn css_wrapper() -> Css {
    css! {"
        max-width: 900px;
        margin: 20px 10px;
    "}
}

/// Size and weight spelled out, not left to the browser.
///
/// The demo ships Tailwind, whose preflight sets `font-size: inherit` and `font-weight:
/// inherit` on every heading and zeroes the margin on `p` - so an `<h1>` here renders as
/// ordinary body text until it is told otherwise.
fn css_title() -> Css {
    css! {"
        font-size: 2em;
        font-weight: bold;
        margin: 0 0 10px 0;
    "}
}

fn css_lead() -> Css {
    css! {"
        max-width: 70ch;
        line-height: 1.5;
        margin: 0 0 12px 0;
        color: #333;
    "}
}

fn css_list() -> Css {
    css! {"
        display: flex;
        flex-direction: column;
        gap: 4px;
        margin: 20px 0;
    "}
}

fn css_entry() -> Css {
    css! {"
        display: flex;
        gap: 10px;
        align-items: baseline;
        padding: 4px 0;
        border-bottom: 1px solid #eee;
    "}
}

/// A fixed width, so the descriptions line up into a column of their own.
fn css_link() -> Css {
    css! {"
        flex-shrink: 0;
        width: 130px;
        font-weight: bold;
    "}
}

fn css_about() -> Css {
    css! {"
        color: #555;
    "}
}
