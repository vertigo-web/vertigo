#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Route {
    Counters,
    Animations,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife,
    Chat,
    Todo,
    DropFile,
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
            "/animations" => Self::Animations,
            "/sudoku" => Self::Sudoku,
            "/input" => Self::Input,
            "/github_explorer" => Self::GithubExplorer,
            "/game_of_life" => Self::GameOfLife,
            "/chat" => Self::Chat,
            "/todo" => Self::Todo,
            "/drop-file" => Self::DropFile,
            _ => Self::NotFound,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Counters => "Counters",
            Self::Animations => "Animations",
            Self::Sudoku => "Sudoku",
            Self::Input => "Input",
            Self::GithubExplorer => "Github Explorer",
            Self::GameOfLife => "Game Of Life",
            Self::Chat => "Chat",
            Self::Todo => "Todo",
            Self::DropFile => "Drop File",
            Self::NotFound => "Not Found",
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
            Self::Counters => "/counters",
            Self::Animations => "/animations",
            Self::Sudoku => "/sudoku",
            Self::Input => "/input",
            Self::GithubExplorer => "/github_explorer",
            Self::GameOfLife { .. } => "/game_of_life",
            Self::Chat => "/chat",
            Self::Todo => "/todo",
            Self::DropFile => "/drop-file",
            Self::NotFound => "/not-found",
        }
        .to_string()
    }
}
