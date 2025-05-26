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
fn test_pub_super() {
    mod sub_module {
        use crate::{self as vertigo, component, dom};

        #[component]
        pub(super) fn Hello<'a>(name: &'a str) {
            dom! {
                <span>"Hello " {name}</span>
            }
        }
    }

    use crate::{self as vertigo, dom, DomNode};

    let ret = dom! {
        <p><sub_module::Hello name={"world"} /></p>
    };

    assert!(matches!(ret, DomNode::Node { node: _ }));
}

#[test]
fn test_pub_crate() {
    mod sub_module {
        pub(super) mod sub_sub_module {
            use crate::{self as vertigo, component, dom};
            #[component]
            pub(crate) fn Hello<'a>(name: &'a str) {
                dom! {
                    <span>"Hello " {name}</span>
                }
            }
        }
    }

    use crate::{self as vertigo, dom, DomNode};

    let ret = dom! {
        <p><sub_module::sub_sub_module::Hello name={"world"} /></p>
    };

    assert!(matches!(ret, DomNode::Node { node: _ }));
}
