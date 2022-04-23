use vertigo::router::HashRouter;
use vertigo::{Driver, VDomComponent};

use super::counters::State as CountersState;
use super::game_of_life;
use super::github_explorer;
use super::input;
use super::route::Route;
use super::sudoku;

#[derive(Clone)]
pub struct State {
    pub driver: Driver,

    pub main: VDomComponent,
    pub counters: VDomComponent,
    pub sudoku: VDomComponent,
    pub input: VDomComponent,
    pub github_explorer: VDomComponent,
    pub game_of_life: VDomComponent,

    pub route: HashRouter<Route>,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let game_of_life = game_of_life::State::component(driver);

        let route = HashRouter::new(driver);

        let state = State {
            driver: driver.clone(),
            main: super::main::MainState::component(driver),
            counters: CountersState::component(driver),
            sudoku: sudoku::Sudoku::component(driver),
            input: input::State::component(driver),
            github_explorer: github_explorer::State::component(driver),
            game_of_life,

            route,
        };

        super::render(state)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
