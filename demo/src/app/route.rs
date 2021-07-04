#[derive(PartialEq, Debug)]
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

impl Route {
    pub fn new(path: &String) -> Route {
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

impl ToString for Route {
    fn to_string(&self) -> String {
        match self {
            Self::Main => "",
            Self::Counters => "counters",
            Self::Sudoku => "sudoku",
            Self::Input => "input",
            Self::GithubExplorer => "github_explorer",
            Self::GameOfLife { .. } => "game_of_life",
            Self::NotFound => "",
        }.to_string()
    }
}

#[macro_export]
macro_rules! navigate_to {
    ($state:ident, $route:ident) => {{
        let $state = $state.clone();
        move || $state.navigate_to(Route::$route)
    }}
}
