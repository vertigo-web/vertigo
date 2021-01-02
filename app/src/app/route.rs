use vertigo::{
    computed::Computed,
    utils::DropResource,
};
use crate::game_of_life::State as GameOfLifeState;

#[derive(PartialEq, Debug)]
pub enum Route {
    Main,
    Counters,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife {
        timer: DropResource,
    },
    NotFound,
}

impl Default for Route {
    fn default() -> Self {
        Self::Main
    }
}

impl Route {
    pub fn new(path: String, game_of_life: &Computed<GameOfLifeState>) -> Route {
        match path.as_str() {
            "" | "/" => Self::Main,
            "counters" => Self::Counters,
            "sudoku" => Self::Sudoku,
            "input" => Self::Input,
            "github_explorer" => Self::GithubExplorer,
            "game_of_life" => {
                let game_of_life = game_of_life.get_value();
                Self::GameOfLife {
                    timer: game_of_life.start_timer(),
                }
            },
            _ => Self::NotFound,
        }
    }

    pub fn is_game_of_life(&self) -> bool {
        if let Route::GameOfLife {..} = self {
            true
        } else {
            false
        }
    }
}

impl ToString for Route {
    fn to_string(&self) -> String {
        match self {
            Self::Main => "",
            Self::Counters => "counters",
            Self::Sudoku => "sudoku",
            Self::Input => "input",
            Self::GithubExplorer => "github_explorer",
            Self::GameOfLife { .. }=> "game_of_life",
            Self::NotFound => "",
        }.to_string()
    }
}
