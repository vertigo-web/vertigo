use vertigo::router::HashRouter;
use vertigo::{VDomComponent};

use super::counters::State as CountersState;
use super::game_of_life;
use super::github_explorer;
use super::input;
use super::route::Route;
use super::sudoku;

#[derive(Clone)]
pub struct State {
    pub main: VDomComponent,
    pub counters: VDomComponent,
    pub sudoku: VDomComponent,
    pub input: VDomComponent,
    pub github_explorer: VDomComponent,
    pub game_of_life: VDomComponent,

    pub route: HashRouter<Route>,
}

impl State {
    pub fn component() -> VDomComponent {
        let game_of_life = game_of_life::State::component();

        let route = HashRouter::new();

        let state = State {
            main: super::main::MainState::component(),
            counters: CountersState::component(),
            sudoku: sudoku::Sudoku::component(),
            input: input::State::component(),
            github_explorer: github_explorer::State::component(),
            game_of_life,

            route,
        };

        super::render(state)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
