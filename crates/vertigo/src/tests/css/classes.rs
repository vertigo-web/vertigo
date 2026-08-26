use crate::{
    DomElement, css,
    dev::{
        command::DriverDomCommand,
        inspect::{DomDebugFragment, log_start, log_take},
    },
    dom,
};

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_explicit_class_attribute() {
    let red_css = css!("color: red;");

    log_start();

    let _el = dom! {
        <div class="flex" tw="flex" css={red_css} />
    };

    let js_log = DomDebugFragment::from_log();

    let el_str = js_log.to_pseudo_html();

    assert_eq!(
        el_str,
        r#"<div class='flex' style='color: red' v-css='red_css' />"#
    );
}

/// `class` then `css` must land where `css` then `class` does.
///
/// An element carries a plain class inline and only builds a merger when a *second* source -
/// css, or a reactive class - turns up. Setting the class first is what makes that promotion
/// happen with a value already written, which has to be carried across; a promotion that
/// dropped it would render the css class on its own.
///
/// Only the builder API reaches this order. The `dom!` macro collects class values and emits
/// them after everything else, so it always goes css-first - which is exactly why the other
/// order needs a test of its own.
///
/// Asserted on the commands rather than on `to_pseudo_html`, which resolves css class names
/// from the `InsertCss` in the same log - and the second half of this test registers none,
/// because the first half already did.
#[test]
fn class_and_css_merge_in_either_order() {
    fn final_class(build: impl FnOnce() -> DomElement) -> String {
        log_start();
        let _el = build();

        log_take()
            .into_iter()
            .rev()
            .find_map(|command| match command {
                DriverDomCommand::SetAttr { name, value, .. } if name.as_str() == "class" => {
                    Some(value)
                }
                _ => None,
            })
            .unwrap_or_default()
    }

    let red = css!("color: red;");

    let css_first = final_class(|| {
        DomElement::new("div")
            .css(red.clone())
            .attr("class", "flex")
    });
    let class_first = final_class(|| {
        DomElement::new("div")
            .attr("class", "flex")
            .css(red.clone())
    });

    assert_eq!(class_first, css_first);
    assert!(
        class_first.starts_with("flex "),
        "a class written before the css must survive the promotion, got: {class_first:?}"
    );
}
