//! What `dom!` accepts as an embedded value.

use std::{borrow::Cow, num::NonZeroU32, rc::Rc};

use crate::{
    self as vertigo, DomNode, EmbedDom,
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

/// Anything printable can be embedded by reference, including a downstream type.
#[test]
fn borrowed_values_embed() {
    let number = 5u32;
    let text = String::from("hi");
    let money = Money(3);

    assert_eq!(html(|| dom! { <div>{&number}</div> }), "<div>5</div>");
    assert_eq!(html(|| dom! { <div>{&text}</div> }), "<div>hi</div>");
    assert_eq!(
        html(|| dom! { <div>{"literal"}</div> }),
        "<div>literal</div>"
    );
    assert_eq!(html(|| dom! { <div>{&money}</div> }), "<div>$3</div>");
}

/// The by-value side is an explicit list precisely so this stays possible: a printable
/// downstream type can render its own DOM rather than being forced into a text node.
impl EmbedDom for Money {
    fn embed(self) -> DomNode {
        dom! { <span>{self.to_string()}</span> }
    }
}

#[test]
fn owned_type_can_define_its_own_embedding() {
    assert_eq!(
        html(|| dom! { <div>{Money(3)}</div> }),
        "<div><span>$3</span></div>"
    );
}
