use std::fmt::Display;

use vertigo::get_driver;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Route {
    #[default]
    Home,
    Counters,
    Styling,
    Sudoku,
    Input,
    GithubExplorer,
    GameOfLife,
    Chat,
    Fetch,
    DropFile,
    Driver,
    JsApiAccess,
    List,
    LazyList,
    WsCollection,
    Svg,
    NotFound,
}

impl Route {
    /// The tabs, in the order the menu shows them.
    ///
    /// The menu is built from this and so is the arrow-key navigation, so the two cannot drift
    /// apart. `NotFound` is deliberately absent - it is somewhere you land, not somewhere to
    /// navigate to.
    pub const ALL: &'static [Route] = &[
        Self::Home,
        Self::Counters,
        Self::Styling,
        Self::Sudoku,
        Self::Input,
        Self::GithubExplorer,
        Self::GameOfLife,
        Self::Chat,
        Self::Fetch,
        Self::DropFile,
        Self::Driver,
        Self::JsApiAccess,
        Self::List,
        Self::LazyList,
        Self::WsCollection,
        Self::Svg,
    ];

    /// The tab `offset` places along from this one, wrapping at both ends.
    pub fn step(&self, offset: isize) -> Route {
        let length = Self::ALL.len() as isize;

        let next = match Self::ALL.iter().position(|route| route == self) {
            Some(current) => (current as isize + offset).rem_euclid(length),
            // Not in the menu, which means `NotFound`: step forward into the first entry and
            // back into the last, rather than pretending it has a position of its own.
            None if offset >= 0 => 0,
            None => length - 1,
        };

        Self::ALL
            .get(next as usize)
            .cloned()
            .unwrap_or(Self::Counters)
    }

    /// One line on what this tab is here to show.
    ///
    /// The landing page is generated from `ALL` and these, so a tab that reaches the menu
    /// cannot be missing from the index, and an index entry cannot outlive its tab.
    pub fn about(&self) -> &'static str {
        match self {
            Self::Home => "This page.",
            Self::Counters => {
                "Value and Computed: one write reaching a counter, a sum, and the sum's double."
            }
            Self::Styling => "The css! macro, an animation, a tooltip, and Tailwind classes.",
            Self::Sudoku => {
                "A wide reactive graph - 81 cells, each deriving its candidates from twenty peers."
            }
            Self::Input => {
                "One Value<String> behind an input, a textarea, and a derived character count."
            }
            Self::GithubExplorer => "LazyCache over a fetch, decoding nested AutoJsJson structs.",
            Self::GameOfLife => {
                "8400 Value<bool> driven by a timer, fanned back into a single population count."
            }
            Self::Chat => "A websocket connection, with each message echoed to every client.",
            Self::Fetch => {
                "LazyCache again, prefetched during SSR so the browser does not ask a second time."
            }
            Self::DropFile => {
                "Files by drag-and-drop or by a file input, both arriving as a DropFileEvent."
            }
            Self::Driver => {
                "get_driver(): cookies, the timezone, a random number, history and the router."
            }
            Self::JsApiAccess => {
                "The js! macro and NodeRef, for reaching what vertigo does not wrap itself."
            }
            Self::List => "render_list and a hand-built dom_element! loop over the same data.",
            Self::LazyList => {
                "LazyListCache: optimistic create, update and delete, rolled back on error."
            }
            Self::WsCollection => {
                "WsCollection - a collection the server pushes, re-queried as you filter it."
            }
            Self::Svg => "Namespaced elements, and the workaround for tags that clash with HTML.",
            Self::NotFound => "Nothing - it is where an unknown address lands.",
        }
    }

    /// The route's own path, before a mount point is applied.
    ///
    /// Separate from `Display` so it can be checked against [`Route::new`] without a driver -
    /// see the round-trip test below. The two are written out independently, and nothing but
    /// that test stops them disagreeing.
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Counters => "/counters",
            Self::Styling => "/styling",
            Self::Sudoku => "/sudoku",
            Self::Input => "/input",
            Self::GithubExplorer => "/github_explorer",
            Self::GameOfLife { .. } => "/game_of_life",
            Self::Chat => "/chat",
            Self::Fetch => "/fetch",
            Self::DropFile => "/drop-file",
            Self::Driver => "/driver",
            Self::JsApiAccess => "/js-api-access",
            Self::List => "/list",
            Self::LazyList => "/lazy-list",
            Self::WsCollection => "/ws-collection",
            Self::Svg => "/svg",
            Self::NotFound => "/not-found",
        }
    }

    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" => Self::Home,
            "/counters" => Self::Counters,
            "/styling" => Self::Styling,
            "/sudoku" => Self::Sudoku,
            "/input" => Self::Input,
            "/github_explorer" => Self::GithubExplorer,
            "/game_of_life" => Self::GameOfLife,
            "/chat" => Self::Chat,
            "/fetch" => Self::Fetch,
            "/drop-file" => Self::DropFile,
            "/driver" => Self::Driver,
            "/js-api-access" => Self::JsApiAccess,
            "/list" => Self::List,
            "/lazy-list" => Self::LazyList,
            "/ws-collection" => Self::WsCollection,
            "/svg" => Self::Svg,
            _ => Self::NotFound,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Counters => "Counters",
            Self::Styling => "Styling",
            Self::Sudoku => "Sudoku",
            Self::Input => "Input",
            Self::GithubExplorer => "Github Explorer",
            Self::GameOfLife => "Game Of Life",
            Self::Chat => "Chat",
            Self::Fetch => "Fetch",
            Self::DropFile => "Drop File",
            Self::Driver => "Driver",
            Self::JsApiAccess => "JS Api Access",
            Self::List => "List",
            Self::LazyList => "Lazy List",
            Self::WsCollection => "WS Collection",
            Self::Svg => "Svg",
            Self::NotFound => "Not Found",
        }
    }
}

impl From<String> for Route {
    fn from(url: String) -> Self {
        let local_url = get_driver().route_from_public(url);
        Route::new(local_url.as_str())
    }
}

impl Display for Route {
    /// The public URL, which is [`Route::path`] with any mount point applied.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&get_driver().route_to_public(self.path()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_moves_along_the_menu() {
        assert_eq!(Route::Home.step(1), Route::Counters);
        assert_eq!(Route::Counters.step(1), Route::Styling);
        assert_eq!(Route::Styling.step(-1), Route::Counters);
    }

    #[test]
    fn stepping_wraps_at_both_ends() {
        assert_eq!(Route::Svg.step(1), Route::Home);
        assert_eq!(Route::Home.step(-1), Route::Svg);
    }

    /// `NotFound` has no place in the menu, so stepping enters at whichever end you came from.
    #[test]
    fn stepping_from_outside_the_menu_enters_it() {
        assert_eq!(Route::NotFound.step(1), Route::Home);
        assert_eq!(Route::NotFound.step(-1), Route::Svg);
    }

    #[test]
    fn a_full_lap_comes_back_to_the_start() {
        let length = Route::ALL.len() as isize;

        for route in Route::ALL {
            assert_eq!(&route.step(length), route);
            assert_eq!(&route.step(-length), route);
        }
    }

    /// `new` and `path` are two hand-written lists of the same thing, so they can disagree -
    /// a typo in one would give a menu link that lands on Not Found.
    #[test]
    fn every_route_round_trips_through_its_path() {
        for route in Route::ALL {
            assert_eq!(
                &Route::new(route.path()),
                route,
                "{route:?} has the path {:?}, which does not route back to it",
                route.path()
            );
        }
    }

    /// Likewise for the other two lists: a copy-pasted arm would show up as a duplicate.
    #[test]
    fn every_tab_is_distinct() {
        for (n, route) in Route::ALL.iter().enumerate() {
            for other in &Route::ALL[n + 1..] {
                assert_ne!(route.path(), other.path(), "{route:?} and {other:?}");
                assert_ne!(route.label(), other.label(), "{route:?} and {other:?}");
                assert_ne!(route.about(), other.about(), "{route:?} and {other:?}");
            }
        }
    }

    /// Every tab is reachable by walking forward from the first.
    #[test]
    fn walking_forward_visits_every_tab() {
        let mut seen = Vec::new();
        let mut route = Route::Home;

        for _ in 0..Route::ALL.len() {
            seen.push(route.clone());
            route = route.step(1);
        }

        assert_eq!(seen, Route::ALL);
        assert_eq!(route, Route::Home, "and ends up back at the start");
    }
}
