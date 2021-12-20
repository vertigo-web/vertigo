use std::cmp::PartialEq;
use vertigo::router::HashRouter;
use vertigo::{Computed, Driver, Value};

use super::counters::State as CountersState;
use super::game_of_life;
use super::github_explorer;
use super::input;
use super::main::MainState;
use super::route::Route;
use super::sudoku;

#[derive(PartialEq)]
pub struct State {
    pub driver: Driver,
    pub route: Value<Route>,

    pub main: Computed<MainState>,
    pub counters: Computed<CountersState>,
    pub sudoku: Computed<sudoku::Sudoku>,
    pub input: Computed<input::State>,
    pub github_explorer: Computed<github_explorer::State>,
    pub game_of_life: Computed<game_of_life::State>,

    hash_router: HashRouter,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let game_of_life = game_of_life::State::new(driver);

        let route: Value<Route> = driver.new_value(Route::new(&driver.get_hash_location()));

        let hash_router = HashRouter::new(driver, route.clone(), {
            let route = route.clone();

            Box::new(move |url: &String| {
                route.set_value(Route::new(url));
            })
        });

        let state = State {
            driver: driver.clone(),
            route,
            main: super::main::MainState::new(driver),
            counters: CountersState::new(driver),
            sudoku: sudoku::Sudoku::new(driver),
            input: input::State::new(driver),
            github_explorer: github_explorer::State::new(driver),
            game_of_life,

            hash_router,
        };

        driver.new_computed_from(state)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
