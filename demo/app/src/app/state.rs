use vertigo::router::HashRouter;
use vertigo::VDomComponent;

use super::counters::State as CountersState;
use super::game_of_life;
use super::github_explorer;
use super::input;
use super::main::AnimationsState;
use super::route::Route;
use super::sudoku::Sudoku;

#[derive(Clone)]
pub struct State {
    pub counters: CountersState,
    pub animations: AnimationsState,
    pub sudoku: Sudoku,
    pub input: input::State,
    pub github_explorer: github_explorer::State,
    pub game_of_life: game_of_life::State,

    pub route: HashRouter<Route>,
}

impl State {
    pub fn component() -> VDomComponent {
        let game_of_life = game_of_life::State::new();

        let route = HashRouter::new();

        let state = State {
            counters: CountersState::new(),
            animations: AnimationsState::new(),
            sudoku: Sudoku::new(),
            input: input::State::new(),
            github_explorer: github_explorer::State::new(),
            game_of_life,

            route,
        };

        let dom = super::render(state);

        VDomComponent::dom(dom)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set(route);
        //log::info!("conn = {}", self.root.all_connections_len());
    }
}
