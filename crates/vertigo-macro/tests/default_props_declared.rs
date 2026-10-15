//! Components declaring `#[default = ...]` props in a crate that doesn't enable
//! `default_field_values`, like a library would. Where the compiler has default field values
//! only behind the feature, `#[component]` emits them under `allow_internal_unstable`; where it
//! has none, it leaves them out. Either way giving every prop keeps working.

use vertigo::{
    component,
    dev::inspect::{DomDebugFragment, log_start},
    dom,
};

#[component]
fn Greeting(
    name: String,
    /// How many times to repeat the name
    #[default = 1]
    times: u32,
    #[default = String::new()] suffix: String,
) {
    let text = format!("{}{suffix}", name.repeat(times as usize));
    dom! { <p>{text}</p> }
}

#[component]
fn Label<'a>(#[default = "none"] text: &'a str) {
    dom! { <span>{text}</span> }
}

#[test]
fn every_prop_given() {
    log_start();
    let _el = dom! { <Greeting name="Hi" times={2u32} suffix="!" /> };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<p v-component='Greeting'>HiHi!</p>");
}

#[test]
fn braces_give_default_trait_value() {
    // `prop={}` is `Default::default()`, not the declared default.
    log_start();
    let _el = dom! { <Greeting name="Hi" times={} suffix={} /> };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<p v-component='Greeting' />");
}

#[test]
fn lifetime_param_with_default() {
    log_start();
    let _el = dom! { <Label text="Hi" /> };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<span v-component='Label'>Hi</span>");
}
