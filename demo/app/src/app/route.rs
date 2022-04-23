#[derive(PartialEq, Debug, Clone)]
pub enum Route {
    Main,
    Counters,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife,
    Chat,
    Todo,
    NotFound,
}

impl Default for Route {
    fn default() -> Self {
        Self::Main
    }
}

impl Route {
    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" => Self::Main,
            "counters" => Self::Counters,
            "sudoku" => Self::Sudoku,
            "input" => Self::Input,
            "github_explorer" => Self::GithubExplorer,
            "game_of_life" => Self::GameOfLife,
            "chat" => Self::Chat,
            "todo" => Self::Todo,
            _ => Self::NotFound,
        }
    }
}

impl From<String> for Route {
    fn from(url: String) -> Self {
        Route::new(url.as_str())
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
            Self::GameOfLife { .. } => "game_of_life",
            Self::Chat => "chat",
            Self::Todo => "todo",
            Self::NotFound => "",
        }
        .to_string()
    }
}
