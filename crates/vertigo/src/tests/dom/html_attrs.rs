use crate as vertigo;
use crate::dom;

#[test]
fn button_on_click() {
    let handler = || ();

    let _ = dom! {
        <button on_click={handler} />
    };
}

#[test]
fn form_on_submit() {
    let handler = || ();

    let _ = dom! {
        <form on_submit={handler} />
    };
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
