use std::cmp::PartialEq;
use vertigo::{
    DomDriver,
    computed::{Computed, Dependencies, Value},
    router::HashRouter,
};

use super::sudoku;
use super::input;
use super::github_explorer;
use super::route::Route;
use super::main::MainState;
use super::counters::State as CountersState;
use super::game_of_life;

#[derive(PartialEq)]
pub struct State {
    root: Dependencies,
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
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {

        let game_of_life = game_of_life::State::new(&root, driver);

        let route: Value<Route> = root.new_value(Route::new(driver.get_hash_location(), &game_of_life));

        let hash_router = HashRouter::new(driver, route.clone(), {
            let route = route.clone();
            let game_of_life = game_of_life.clone();

            Box::new(move |url: String|{
                route.set_value(Route::new(url, &game_of_life));
            })
        });

        let state = State {
            root: root.clone(),
            route,
            main: super::main::MainState::new(&root),
            counters: CountersState::new(&root),
            sudoku: sudoku::Sudoku::new(root),
            input: input::State::new(&root),
            github_explorer: github_explorer::State::new(&root, driver),
            game_of_life,

            hash_router,
        };

        root.new_computed_from(state)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
