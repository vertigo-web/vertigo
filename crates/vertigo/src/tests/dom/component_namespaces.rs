#[test]
/// This test checks if component can be used when not in local scope
/// (whole name does not start with capital letter)
fn test_namespaces() {
    use crate::dev::inspect::{DomDebugFragment, log_start};
    use crate::{self as vertigo, dom};

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

    log_start();
    let _el = dom! {
        <my_module::inner::Hello name={"world"} />
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<span v-component='my_module::inner::Hello'>Hello world</span>"
    );
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

    use crate::dev::inspect::{DomDebugFragment, log_start};
    use crate::{self as vertigo, dom};

    log_start();
    let _el = dom! {
        <p><sub_module::Hello name={"world"} /></p>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<p><span v-component='sub_module::Hello'>Hello world</span></p>"
    );
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

    use crate::dev::inspect::{DomDebugFragment, log_start};
    use crate::{self as vertigo, dom};

    log_start();
    let _el = dom! {
        <p><sub_module::sub_sub_module::Hello name={"world"} /></p>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<p><span v-component='sub_module::sub_sub_module::Hello'>Hello world</span></p>"
    );
}
