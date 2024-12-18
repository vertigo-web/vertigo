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
/// This test checks if component can be used when not in local scope
/// (whole name does not start with capital letter)
fn test_namespaces() {
    use crate::{self as vertigo, dom, DomNode};

    mod my_module {
        pub mod inner {
            use crate::{self as vertigo, component, dom};
            #[component]
            pub fn Hello(name: String) {
                dom! {
                    <span>"Hello " {name}</span>
                }
            }
        }
    }

    let ret = dom! {
        <my_module::inner::Hello name={"world"} />
    };

    match ret {
        DomNode::Node { node } => {
            match node.get_children().pop_back() {
                // If node has text child, then component "Hello" was embedded correctly
                Some(child) => match child {
                    DomNode::Text { node: _ } => {}
                    _ => panic!("Expected text child"),
                },
                _ => panic!("Expected child node"),
            }
        }
        _ => panic!("Expected DomNode::Node"),
    }
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
