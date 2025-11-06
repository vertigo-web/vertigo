use crate as vertigo;
use crate::dom;
use crate::inspect::{log_start, log_take};

#[test]
fn button_on_click() {
    let handler = |_| ();

    log_start();

    let _ = dom! {
        <button on_click={handler} />
    };

    log_take();
}

#[test]
fn form_on_submit() {
    let handler = || ();

    log_start();

    let _ = dom! {
        <form on_submit={handler} />
    };

    log_take();
}

#[test]
fn no_keys() {
    let src = "cat.png";
    let alt = "Not dog";

    let _ = dom! {
        <img {src} {alt} />
    };
}

#[test]
fn default_values() {
    let _ = dom! {
        <img
            src={Default::default()}
            alt={}
        />
    };
}

#[test]
fn references() {
    let src = "cat.png";
    let alt = "Not dog";

    let _ = dom! {
        <img src={&src} {&alt} />
    };
}
