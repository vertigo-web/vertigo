use crate::dev::inspect::{log_start, DomDebugFragment};

#[test]
fn test_if_lifetimes_allowed() {
    use crate::{self as vertigo, component, dom, DomNode};

    #[component]
    fn Hello<'a>(name: &'a str) {
        dom! {
            <span>"Hello " {name}</span>
        }
    }

    let ret = dom! {
        <p><Hello name={"world"} /></p>
    };

    assert!(matches!(ret, DomNode::Node { node: _ }));
}

#[test]
fn test_generics() {
    use std::fmt::Display;

    use crate::{self as vertigo, component, dom, DomNode};

    #[component]
    fn Hello<T: Display>(name: T) {
        dom! {
            <span>"Hello " {name}</span>
        }
    }

    let ret = dom! {
        <p><Hello name={&"world"} /></p>
    };

    assert!(matches!(ret, DomNode::Node { node: _ }));
}

#[test]
fn test_if_docstrings_allowed() {
    use crate::{self as vertigo, component, dom, DomNode};

    #[component]
    fn Hello<'a>(
        /// Name of the person you want to greet
        name: &'a str,
    ) {
        dom! {
            <span>"Hello " {name}</span>
        }
    }

    let ret = dom! {
        <p><Hello name={"world"} /></p>
    };

    assert!(matches!(ret, DomNode::Node { node: _ }));
}

#[test]
fn test_manual_mounting() {
    use crate::{self as vertigo, component, dom};

    #[component]
    fn Hello<'a>(name: &'a str) {
        dom! {
            <span>"Hello " {name}</span>
        }
    }

    log_start();
    let _el1 = Hello { name: "John" }.mount();
    let el1_str = DomDebugFragment::from_log().to_pseudo_html();

    log_start();
    let _el2 = Hello { name: "John" }.into_component().mount();
    let el2_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(el1_str, "<span>Hello John</span>");
    assert_eq!(el1_str, el2_str);
}

#[test]
fn test_mutable_parameter() {
    use crate::{self as vertigo, component, dom};

    #[component]
    fn Hello<'a, T: Into<String>>(prefix: T, mut name: String, surname: &'a mut String) {
        if name == "John" {
            name = "Jack".to_string();
        }

        if surname == "Johnson" {
            *surname = "Jackson".to_string();
        }

        dom! {
            <span>"Hello " {prefix.into()} " " {name} " " {surname}</span>
        }
    }

    log_start();
    let prefix = "Mr.";
    let _el1 = dom! { <Hello {&prefix} name="John" surname={&&mut "Johnson".to_string()} /> };
    let el1_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el1_str,
        "<span v-component='Hello'>Hello Mr. Jack Jackson</span>"
    );
}

#[test]
fn test_mutable_ref_parameter() {
    use crate::{self as vertigo, component, dom};

    #[component]
    fn Hello<'a>(name: &'a mut String) {
        *name = "Jack".to_string();

        dom! {
            <span>"Hello " {name}</span>
        }
    }

    log_start();
    let _el1 = Hello {
        name: &mut "John".to_string(),
    }
    .mount();
    let el1_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(el1_str, "<span>Hello Jack</span>");
}
