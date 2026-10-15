//! Leaving out props with `{..}`. The crate doing it needs default field values, so this runs
//! only where the compiler has them (see build.rs), with the feature enabled while unstable.
#![cfg(vertigo_default_props)]
#![cfg_attr(vertigo_default_props_unstable, feature(default_field_values))]

use vertigo::{
    DomNode, component,
    dev::inspect::{DomDebugFragment, log_start},
    dom,
};

#[component]
fn Greeting(name: String, #[default = 1] times: u32, #[default = String::new()] suffix: String) {
    let text = format!("{}{suffix}", name.repeat(times as usize));
    dom! { <p>{text}</p> }
}

#[component]
fn Label<'a>(#[default = "none"] text: &'a str) {
    dom! { <span>{text}</span> }
}

fn render(node: impl FnOnce() -> DomNode) -> String {
    log_start();
    let _node = node();
    DomDebugFragment::from_log().to_pseudo_html()
}

#[test]
fn left_out_props_take_defaults() {
    let html = render(|| dom! { <Greeting name="Hi" {..} /> });
    assert_eq!(html, "<p v-component='Greeting'>Hi</p>");
}

#[test]
fn given_props_override_defaults() {
    let html = render(|| dom! { <Greeting name="Hi" times={3u32} {..} /> });
    assert_eq!(html, "<p v-component='Greeting'>HiHiHi</p>");
}

#[test]
fn rest_with_every_prop_given() {
    let html = render(|| dom! { <Greeting name="Hi" times={2u32} suffix="!" {..} /> });
    assert_eq!(html, "<p v-component='Greeting'>HiHi!</p>");
}

#[test]
fn child_component_with_rest() {
    let html = render(|| dom! { <div><Greeting name="Hi" {..} /></div> });
    assert_eq!(html, "<div><p v-component='Greeting'>Hi</p></div>");
}

#[test]
fn lifetime_param_default() {
    let html = render(|| dom! { <Label {..} /> });
    assert_eq!(html, "<span v-component='Label'>none</span>");
}
