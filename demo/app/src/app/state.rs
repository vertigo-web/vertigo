use std::cmp::PartialEq;
use vertigo::router::HashRouter;
use vertigo::{Driver, Value, VDomComponent};

use super::counters::State as CountersState;
use super::game_of_life;
use super::github_explorer;
use super::input;
use super::route::Route;
use super::sudoku;

#[derive(PartialEq)]
pub struct State {
    pub driver: Driver,
    pub route: Value<Route>,

    pub main: VDomComponent,
    pub counters: VDomComponent,
    pub sudoku: VDomComponent,
    pub input: VDomComponent,
    pub github_explorer: VDomComponent,
    pub game_of_life: VDomComponent,

    hash_router: HashRouter,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let game_of_life = game_of_life::State::component(driver);

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
            main: super::main::MainState::component(driver),
            counters: CountersState::component(driver),
            sudoku: sudoku::Sudoku::component(driver),
            input: input::State::component(driver),
            github_explorer: github_explorer::State::component(driver),
            game_of_life,

            hash_router,
        };

        driver.bind_render(state, super::render)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
