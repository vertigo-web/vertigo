//! A static asset, styled two ways.
//!
//! `include_static!` hashes the file at build time and gives back the URL it will be served
//! from, so the image is part of the binary's manifest rather than a path to keep in step by
//! hand. The two copies show `Css` being composed: a shared base, then `+` for one and
//! `push_str` for the other.

use vertigo::{component, css, dom, include_static};

#[component]
pub fn Images() {
    let path = include_static!("./counter.webp");

    let center_base = css! {"
        border: 1px solid black;
        padding: 1px;
        margin: 0 auto;
        display: block;

        cursor: pointer;

        transition: all .2s ease-in-out;
    "};

    let center_css = center_base.clone()
        + css! {"
        box-shadow: 4px 4px 4px #444, 8px 8px 4px #666, 12px 12px 4px #888;

        :hover {
            transform: scale(1.1);
        }
    "};

    let center_css2 = center_base.push_str("
        box-shadow: 4px 4px 4px rgba(0, 0, 0, 0.5), 8px 8px 4px rgba(0, 0, 0, 0.4), 12px 12px 4px rgba(0, 0, 0, 0.3);
        :hover {
            transform: scale(1.5);
            box-shadow: 54px 54px 14px rgba(0, 0, 0, 0.3), 58px 58px 14px rgba(0, 0, 0, 0.2), 62px 62px 14px rgba(0, 0, 0, 0.1);
        }
    ");

    dom! {
        <div>
            <p>"A static asset through include_static!, with two different shadows on hover:"</p>
            <img css={center_css} src={&path} />
            <img css={center_css2} src={path} />
        </div>
    }
}
