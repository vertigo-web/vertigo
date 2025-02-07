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
