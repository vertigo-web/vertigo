use vertigo::DomElement;
use vertigo::Value;
use vertigo::router::Router;

use super::counters;
use super::game_of_life;
use super::github_explorer;
use super::route::Route;
use super::sudoku::SudokuState;

#[derive(Clone)]
pub struct State {
    pub counters: counters::State,
    pub sudoku: SudokuState,
    pub input: Value<String>,
    pub github_explorer: github_explorer::State,
    pub game_of_life: game_of_life::State,

    pub route: Router<Route>,
}

impl State {
    pub fn new() -> Self {
        State {
            counters: counters::State::new(),
            sudoku: SudokuState::new(),
            input: Value::default(),
            github_explorer: github_explorer::State::new(),
            game_of_life: game_of_life::State::new(),
            route: Router::new_history_router(),
        }
    }

    pub fn render(&self) -> DomElement {
        super::render(self)
    }
}
