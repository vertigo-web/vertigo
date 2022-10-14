use vertigo::DomElement;
use vertigo::router::HashRouter;

use super::counters;
use super::game_of_life;
use super::github_explorer;
use super::input::MyInput;
use super::route::Route;
use super::sudoku::SudokuState;

#[derive(Clone)]
pub struct State {
    pub counters: counters::State,
    pub sudoku: SudokuState,
    pub input: MyInput,
    pub github_explorer: github_explorer::State,
    pub game_of_life: game_of_life::State,

    pub route: HashRouter<Route>,
}

impl State {
    pub fn component() -> DomElement {
        let game_of_life = game_of_life::State::new();

        let route = HashRouter::new();

        let state = State {
            counters: counters::State::new(),
            sudoku: SudokuState::new(),
            input: MyInput::default(),
            github_explorer: github_explorer::State::new(),
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
