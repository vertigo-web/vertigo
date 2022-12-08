use vertigo::DomElement;
use vertigo::router::Router;

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

    pub route: Router<Route>,
}

impl State {
    pub fn new() -> Self {
        State {
            counters: counters::State::new(),
            sudoku: SudokuState::new(),
            input: MyInput::default(),
            github_explorer: github_explorer::State::new(),
            game_of_life: game_of_life::State::new(),
            route: Router::new_history_router(),
        }
    }

    pub fn render(&self) -> DomElement {
        super::render(self)
    }
}
