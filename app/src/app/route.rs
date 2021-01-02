use alloc::{
    string::String,
};
#[derive(PartialEq, Clone, Debug)]
pub enum Route {
    Main,
    Counters,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife,
    NotFound,
}

impl Default for Route {
    fn default() -> Self {
        Self::Main
    }
}

impl From<String> for Route {
    fn from(path: String) -> Self {
        match path.as_str() {
            "" | "/" => Self::Main,
            "counters" => Self::Counters,
            "sudoku" => Self::Sudoku,
            "input" => Self::Input,
            "github_explorer" => Self::GithubExplorer,
            "game_of_life" => Self::GameOfLife,
            _ => Self::NotFound,
        }
    }
}

impl Into<String> for Route {
    fn into(self) -> String {
        match self {
            Self::Main => "",
            Self::Counters => "counters",
            Self::Sudoku => "sudoku",
            Self::Input => "input",
            Self::GithubExplorer => "github_explorer",
            Self::GameOfLife => "game_of_life",
            Self::NotFound => "",
        }.into()
    }
}
