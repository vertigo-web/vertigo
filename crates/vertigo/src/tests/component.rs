#[test]
fn test_lifetimes() {
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
