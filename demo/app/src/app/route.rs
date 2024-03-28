use std::fmt::Display;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Route {
    Counters,
    Styling,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife,
    Chat,
    Todo,
    DropFile,
    JsApiAccess,
    NotFound,
}

impl Default for Route {
    fn default() -> Self {
        Self::Counters
    }
}

impl Route {
    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" | "/counters" => Self::Counters,
            "/styling" => Self::Styling,
            "/sudoku" => Self::Sudoku,
            "/input" => Self::Input,
            "/github_explorer" => Self::GithubExplorer,
            "/game_of_life" => Self::GameOfLife,
            "/chat" => Self::Chat,
            "/todo" => Self::Todo,
            "/drop-file" => Self::DropFile,
            "/js-api-access" => Self::JsApiAccess,
            _ => Self::NotFound,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Counters => "Counters",
            Self::Styling => "Styling",
            Self::Sudoku => "Sudoku",
            Self::Input => "Input",
            Self::GithubExplorer => "Github Explorer",
            Self::GameOfLife => "Game Of Life",
            Self::Chat => "Chat",
            Self::Todo => "Todo",
            Self::DropFile => "Drop File",
            Self::JsApiAccess => "JS Api Access",
            Self::NotFound => "Not Found",
        }
    }
}

impl From<String> for Route {
    fn from(url: String) -> Self {
        Route::new(url.as_str())
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Counters => "/counters",
            Self::Styling => "/styling",
            Self::Sudoku => "/sudoku",
            Self::Input => "/input",
            Self::GithubExplorer => "/github_explorer",
            Self::GameOfLife { .. } => "/game_of_life",
            Self::Chat => "/chat",
            Self::Todo => "/todo",
            Self::DropFile => "/drop-file",
            Self::JsApiAccess => "/js-api-access",
            Self::NotFound => "/not-found",
        };
        f.write_str(str)
    }
}
