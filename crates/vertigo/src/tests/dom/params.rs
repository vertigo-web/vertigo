use crate as vertigo;
use crate::dev::inspect::{DomDebugFragment, log_start};
use crate::{component, dom};

#[component]
fn MyComponent(param_1: i32, param_2: i32) {
    dom! {
        <div>
            <p>{param_1}</p>
            <p>{param_2}</p>
        </div>
    }
}

#[test]
fn component_static_param() {
    log_start();
    let _el = dom! {
        <div>
            <MyComponent
                param_1={1}
                param_2={2}
            />
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<div><div v-component='MyComponent'><p>1</p><p>2</p></div></div>"
    );
}

#[test]
fn component_variable_params() {
    let param_1 = 1;
    let param_2 = 2;

    log_start();
    let _el = dom! {
        <div>
            <MyComponent
                param_1={param_1}
                param_2={&param_2}
            />
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<div><div v-component='MyComponent'><p>1</p><p>2</p></div></div>"
    );
}

#[test]
fn component_params_with_inferred_names() {
    let param_1 = 1;
    let param_2 = 2;

    log_start();
    let _el = dom! {
        <div>
            <MyComponent
                {param_1}
                {&param_2}
            />
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<div><div v-component='MyComponent'><p>1</p><p>2</p></div></div>"
    );
}

#[test]
fn component_params_with_default_values() {
    log_start();
    let _el = dom! {
        <div>
            <MyComponent
                param_1={Default::default()}
                param_2={}
            />
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<div><div v-component='MyComponent'><p>0</p><p>0</p></div></div>"
    );
}

#[test]
fn component_params_with_blocks() {
    let param_1 = 1;
    let param_2 = 2;

    log_start();
    let _el = dom! {
        <div>
            <MyComponent
                param_1={
                    let mut x = param_1 + param_2;
                    x += 1;
                    x
                }
                {
                    let mut param_2 = param_1 + param_2;
                    param_2 += 1;
                    param_2
                }
            />
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<div><div v-component='MyComponent'><p>4</p><p>4</p></div></div>"
    );
}
