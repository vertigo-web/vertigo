use crate as vertigo;
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
    let _ = dom! {
        <div>
            <MyComponent
                param_1={1}
                param_2={2}
            />
        </div>
    };
}

#[test]
fn component_variable_params() {
    let param_1 = 1;
    let param_2 = 2;

    let _ = dom! {
        <div>
            <MyComponent
                param_1={param_1}
                param_2={&param_2}
            />
        </div>
    };
}

#[test]
fn component_params_with_inferred_names() {
    let param_1 = 1;
    let param_2 = 2;

    let _ = dom! {
        <div>
            <MyComponent
                {param_1}
                {&param_2}
            />
        </div>
    };
}

#[test]
fn component_params_with_default_values() {
    let _ = dom! {
        <div>
            <MyComponent
                param_1={Default::default()}
                param_2={}
            />
        </div>
    };
}
