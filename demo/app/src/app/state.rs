use vertigo::router::Router;
use vertigo::DomNode;
use vertigo::Value;

use super::game_of_life;
use super::route::Route;
use super::sudoku::SudokuState;

#[derive(Clone)]
pub struct State {
    pub ws_chat: Option<String>,
    pub sudoku: SudokuState,
    pub input: Value<String>,
    pub game_of_life: game_of_life::State,

    pub route: Router<Route>,
}

impl State {
    pub fn new(ws_chat: Option<String>) -> Self {
        State {
            ws_chat,
            sudoku: SudokuState::new(),
            input: Value::default(),
            game_of_life: game_of_life::State::new(),
            route: Router::new_history_router(),
        }
    }

    pub fn render(&self) -> DomNode {
        super::render(self)
    }
}
