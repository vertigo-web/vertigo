use crate as vertigo;
use crate::dev::inspect::{DomDebugFragment, log_start};
use crate::{DropFileEvent, dom};

#[test]
fn button_on_click() {
    let handler = |_| ();

    log_start();

    let _el = dom! {
        <button on_click={handler} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<button click=1 />");
}

#[test]
fn form_on_submit() {
    let handler = || ();

    log_start();

    let _el = dom! {
        <form on_submit={handler} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<form submit=1 />");
}

#[test]
fn no_keys() {
    let src = "cat.png";
    let alt = "Not dog";

    log_start();

    let _el = dom! {
        <img {src} {alt} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<img alt='Not dog' src='cat.png' />");
}

#[test]
fn default_values() {
    log_start();

    let _el = dom! {
        <img
            src={Default::default()}
            alt={}
        />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<img alt='' src='' />");
}

#[test]
fn input_on_change_file() {
    let handler = |_: DropFileEvent| ();

    log_start();

    let _el = dom! {
        <input type="file" on_change_file={handler} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<input type='file' change_file=1 />");
}

#[test]
fn input_on_change_file_accept() {
    let handler = |_: DropFileEvent| ();
    let accept = "image/*";

    log_start();

    let _el = dom! {
        <input type="file" {accept} on_change_file={handler} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<input accept='image/*' type='file' change_file=1 />");
}

#[test]
fn references() {
    let src = "cat.png";
    let alt = "Not dog";

    log_start();

    let _el = dom! {
        <img src={&src} {&alt} />
    };

    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<img alt='Not dog' src='cat.png' />");
}
