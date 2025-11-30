use crate::{
    css,
    dev::inspect::{log_start, DomDebugFragment},
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
        r#"<div class='flex display-flex' style='color: red' v-css='red_css' />"#
    );
}
