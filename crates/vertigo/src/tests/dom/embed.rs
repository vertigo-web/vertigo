//! What `dom!` accepts as an embedded value.

use std::{borrow::Cow, num::NonZeroU32, rc::Rc};

use crate::{
    self as vertigo, Computed, DomDisplay, DomNode, EmbedDom, Value,
    dev::inspect::{DomDebugFragment, log_start},
    dom,
};

/// A downstream type: printable, but rendering is its own business.
struct Money(u32);

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

/// A downstream type which just prints, and says so.
#[derive(Clone, PartialEq)]
struct Route(u32);

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/post/{}", self.0)
    }
}

impl DomDisplay for Route {}

fn html(node: impl FnOnce() -> DomNode) -> String {
    log_start();
    let _root = node();
    DomDebugFragment::from_log().to_pseudo_html()
}

#[test]
fn owned_values_embed() {
    assert_eq!(html(|| dom! { <div>{5u32}</div> }), "<div>5</div>");
    assert_eq!(html(|| dom! { <div>{true}</div> }), "<div>true</div>");
    assert_eq!(
        html(|| dom! { <div>{String::from("hi")}</div> }),
        "<div>hi</div>"
    );
    assert_eq!(
        html(|| dom! { <div>{NonZeroU32::MIN}</div> }),
        "<div>1</div>"
    );
    assert_eq!(
        html(|| dom! { <div>{Cow::Borrowed("cow")}</div> }),
        "<div>cow</div>"
    );
    assert_eq!(html(|| dom! { <div>{Rc::new(9u32)}</div> }), "<div>9</div>");
}

#[test]
fn borrowed_values_embed() {
    let number = 5u32;
    let text = String::from("hi");

    assert_eq!(html(|| dom! { <div>{&number}</div> }), "<div>5</div>");
    assert_eq!(html(|| dom! { <div>{&text}</div> }), "<div>hi</div>");
    assert_eq!(
        html(|| dom! { <div>{"literal"}</div> }),
        "<div>literal</div>"
    );
}

/// One `impl DomDisplay` covers both directions, and the attribute side with it.
#[test]
fn a_dom_display_type_embeds_owned_and_borrowed() {
    let route = Route(7);

    assert_eq!(html(|| dom! { <div>{&route}</div> }), "<div>/post/7</div>");
    assert_eq!(
        html(|| dom! { <div>{Route(8)}</div> }),
        "<div>/post/8</div>"
    );
    assert_eq!(
        html(|| dom! { <a href={&route}>"x"</a> }),
        "<a href='/post/7'>x</a>"
    );
}

/// Printing is opt-in precisely so this stays possible: a printable downstream type can
/// render its own DOM rather than being forced into a text node. `Money` implements
/// `Display` but not `DomDisplay`, so this impl does not collide with anything.
impl EmbedDom for Money {
    fn embed(self) -> DomNode {
        dom! { <span>{self.to_string()}</span> }
    }
}

#[test]
fn a_type_can_define_its_own_embedding() {
    assert_eq!(
        html(|| dom! { <div>{Money(3)}</div> }),
        "<div><span>$3</span></div>"
    );
}

/// The foreign types the orphan rule leaves to vertigo. An application depending on both
/// vertigo and chrono still cannot write `impl DomDisplay for NaiveDate` itself - E0117 - so
/// the `chrono` feature carries it, in text position and attribute position alike.
#[cfg(feature = "chrono")]
#[test]
fn chrono_types_embed() {
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

    // `unwrap_or_default` rather than `unwrap`: the workspace denies both `unwrap_used` and
    // `expect_used`, and a wrong date fails the assert below just as loudly.
    let date = NaiveDate::from_ymd_opt(2026, 8, 30).unwrap_or_default();
    let time = NaiveTime::from_hms_opt(14, 3, 11).unwrap_or_default();
    let stamp = Utc.from_utc_datetime(&date.and_time(time));

    assert_eq!(
        html(|| dom! { <div>{&date}</div> }),
        "<div>2026-08-30</div>"
    );
    assert_eq!(html(|| dom! { <div>{time}</div> }), "<div>14:03:11</div>");
    assert_eq!(
        html(|| dom! { <div>{date.and_time(time)}</div> }),
        "<div>2026-08-30 14:03:11</div>"
    );
    assert_eq!(
        html(|| dom! { <div>{stamp}</div> }),
        "<div>2026-08-30 14:03:11 UTC</div>"
    );
    assert_eq!(
        html(|| dom! { <time datetime={&date}>"then"</time> }),
        "<time datetime='2026-08-30'>then</time>"
    );
}

/// Same reasoning as the chrono types above.
#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_embeds() {
    let price = rust_decimal::Decimal::new(1234, 2);

    assert_eq!(html(|| dom! { <div>{&price}</div> }), "<div>12.34</div>");
    assert_eq!(
        html(|| dom! { <data value={price}>"cost"</data> }),
        "<data value='12.34'>cost</data>"
    );
}

/// A reactive wrapper embeds anything printable, opted in or not - it can only ever be text.
#[test]
fn reactive_values_embed() {
    let text = Value::new(String::from("hi"));
    let number: Computed<u32> = Value::new(5u32).to_computed();
    let route = Value::new(Route(7));

    assert_eq!(html(|| dom! { <div>{text}</div> }), "<div>hi</div>");
    assert_eq!(html(|| dom! { <div>{number}</div> }), "<div>5</div>");
    assert_eq!(html(|| dom! { <div>{route}</div> }), "<div>/post/7</div>");
}
