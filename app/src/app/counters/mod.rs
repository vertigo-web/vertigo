use std::cmp::PartialEq;
use vertigo::{
    node_attr,
    VDomNode,
    computed::{
        Computed,
        Dependencies
    }
};

mod simple_counter;

#[derive(PartialEq)]
pub struct State {
    pub counter1: Computed<simple_counter::State>,
    pub counter2: Computed<simple_counter::State>,
    pub counter3: Computed<simple_counter::State>,
    pub counter4: Computed<simple_counter::State>,

    pub suma: Computed<u32>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        let counter1 = simple_counter::State::new(&root);
        let counter2 = simple_counter::State::new(&root);
        let counter3 = simple_counter::State::new(&root);
        let counter4 = simple_counter::State::new(&root);

        let suma = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            root.from(move || {
                let value1 = *counter1.get_value().counter.get_value();
                let value2 = *counter2.get_value().counter.get_value();
                let value3 = *counter3.get_value().counter.get_value();
                let value4 = *counter4.get_value().counter.get_value();

                value1 + value2 + value3 + value4
            })
        };

        root.new_computed_from(State {
            counter1,
            counter2,
            counter3,
            counter4,
            suma,
        })
    }
}

fn render_suma(state: &Computed<State>) -> VDomNode {
    use node_attr::{build_node, text};

    let state = state.get_value();

    let suma = state.suma.get_value();

    build_node("div", vec!(
        text(format!("suma = {}", suma))
    ))
}

pub fn render(state: &Computed<State>) -> VDomNode {
    use node_attr::{build_node, component};

    let suma = component(state.clone(), render_suma);

    let state = state.get_value();

    build_node("div", vec!(
        component(state.counter1.clone(), simple_counter::render),
        component(state.counter2.clone(), simple_counter::render),
        component(state.counter3.clone(), simple_counter::render),
        component(state.counter4.clone(), simple_counter::render),
        suma,
    ))
}