use std::cmp::PartialEq;
use vertigo::{
    DomDriver,
    computed::{Computed, Dependencies, Value},
    router::HashRouter,
};

use crate::{game_of_life, simple_counter};
use crate::sudoku;
use crate::input;
use crate::github_explorer;
use super::route::Route;

#[derive(PartialEq)]
pub struct State {
    pub route: Value<Route>,

    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: Computed<simple_counter::State>,
    pub counter2: Computed<simple_counter::State>,
    pub counter3: Computed<simple_counter::State>,
    pub counter4: Computed<simple_counter::State>,

    pub suma: Computed<u32>,
    pub sudoku: Computed<sudoku::Sudoku>,
    pub input: Computed<input::State>,
    pub github_explorer: Computed<github_explorer::State>,
    pub game_of_life: Computed<game_of_life::State>,

    hash_router: HashRouter,
}

impl State {
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {
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

        let route: Value<Route> = root.new_value(driver.get_hash_location().into());

        let hash_router = HashRouter::new(driver, route.clone(), None);

        let state = State {
            route,
            value: root.new_value(33),
            at: root.new_value(999),
            counter1,
            counter2,
            counter3,
            counter4,
            suma,
            sudoku: sudoku::Sudoku::new(root),
            input: input::State::new(&root),
            github_explorer: github_explorer::State::new(&root, driver),
            game_of_life: game_of_life::State::new(&root),

            hash_router,
        };

        root.new_computed_from(state)
    }

    pub fn increment(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr - 1);
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
    }
}
