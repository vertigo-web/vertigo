//! What the demo walk needs beyond the helpers of `vertigo-testing`'s `Ctx`.
//!
//! Every check returns an error rather than panicking, so the harness still gets to save the
//! screenshot, page and console log of a failure.

use std::fmt::{Debug, Display};

use vertigo_testing::{prelude::*, thirtyfour::WebElement};

use crate::Ctx;

/// `assert_eq!` as an error.
pub fn check_eq<L, R>(left: L, right: R, what: impl Display) -> Result<()>
where
    L: PartialEq<R> + Debug,
    R: Debug,
{
    ensure!(left == right, "{what}\n  left: {left:?}\n right: {right:?}");
    Ok(())
}

/// Every element matching `selector`, waiting for at least one to exist first.
pub async fn find_all(ctx: &Ctx, selector: &str) -> Result<Vec<WebElement>> {
    ctx.wait_for(&format!("at least one {selector:?}"), async || {
        let found = ctx.find_all(By::Css(selector)).await?;
        Ok((!found.is_empty()).then_some(found))
    })
    .await
}

/// How many elements match `selector` right now.
pub async fn count(ctx: &Ctx, selector: &str) -> usize {
    ctx.find_all(By::Css(selector))
        .await
        .map(|found| found.len())
        .unwrap_or(0)
}

/// Waits until `selector` matches exactly `count` elements.
pub async fn wait_for_count(ctx: &Ctx, selector: &str, expected: usize) -> Result<()> {
    ctx.wait_for(
        &format!("{expected} element(s) matching {selector:?}"),
        async || Ok((count(ctx, selector).await == expected).then_some(())),
    )
    .await
}

/// The element matching `selector` whose *own* text is exactly `text`.
///
/// Own text means this element's direct text-node children, and nothing a descendant
/// contributes. That distinction is what makes the demo's nested clickable divs addressable:
/// `<div on_click>"outer click"<br/><button>"Inner click"</button></div>` owns "outer click"
/// while its rendered text is both. Neither the element's text (which is the whole subtree) nor
/// an XPath `text()` (which is the *first* text node, and so misses `"post = " {title}`) says
/// what is wanted here.
///
/// Matched in one script rather than a round-trip per candidate - some tabs put hundreds of
/// divs on the page.
pub async fn find_by_text(ctx: &Ctx, selector: &str, text: &str) -> Result<WebElement> {
    const SCRIPT: &str = r#"
        const [selector, wanted] = arguments;
        return Array.from(document.querySelectorAll(selector)).find((node) =>
            Array.from(node.childNodes)
                .filter((child) => child.nodeType === Node.TEXT_NODE)
                .map((child) => child.textContent)
                .join('')
                .trim() === wanted
        ) || null;
    "#;

    let found = ctx
        .wait_for(
            &format!("a {selector} owning the text {text:?}"),
            async || {
                let value = ctx
                    .js_with(SCRIPT, vec![json!(selector), json!(text)])
                    .await?;
                if value.is_null() {
                    return Ok(None);
                }
                Ok(Some(WebElement::from_json(
                    value,
                    ctx.driver.handle().clone(),
                )?))
            },
        )
        .await;

    match found {
        Ok(element) => Ok(element),
        // Says what the page *does* offer, so a renamed label reads as a rename rather than as
        // a mystery.
        Err(err) => {
            let candidates = own_texts(ctx, selector).await;
            Err(err.context(format!("{selector} elements own: {candidates:?}")))
        }
    }
}

/// Every own-text on the page for `selector`, for a failure message.
async fn own_texts(ctx: &Ctx, selector: &str) -> Vec<String> {
    const SCRIPT: &str = r#"
        const [selector] = arguments;
        return Array.from(document.querySelectorAll(selector)).map((node) =>
            Array.from(node.childNodes)
                .filter((child) => child.nodeType === Node.TEXT_NODE)
                .map((child) => child.textContent)
                .join('')
                .trim()
        );
    "#;

    ctx.js_with(SCRIPT, vec![json!(selector)])
        .await
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .filter(|text| !text.is_empty())
        .collect()
}

/// Clicks the element matching `selector` whose own text is exactly `text`.
pub async fn click_by_text(ctx: &Ctx, selector: &str, text: &str) -> Result<()> {
    find_by_text(ctx, selector, text)
        .await?
        .click()
        .await
        .with_context(|| format!("clicking the {selector} reading {text:?} failed"))
}
