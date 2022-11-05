use vertigo::router::HashRouter;
use vertigo::{bind, dom, DomElement};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Route {
    Page1,
    Page2,
    NotFound,
}

impl Route {
    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" | "/page1" => Self::Page1,
            "page2" => Self::Page2,
            _ => Self::NotFound,
        }
    }
}

impl ToString for Route {
    fn to_string(&self) -> String {
        match self {
            Self::Page1 => "",
            Self::Page2 => "page2",
            Self::NotFound => "",
        }
        .to_string()
    }
}

impl From<String> for Route {
    fn from(url: String) -> Self {
        Route::new(url.as_str())
    }
}

#[derive(Clone)]
pub struct App {
    pub route: HashRouter<Route>,
}

impl App {
    pub fn new() -> Self {
        let route = HashRouter::new();

        Self {
            route,
        }
    }

    pub fn mount(self) -> DomElement {
        let state = self;

        let navigate_to_page1 = bind!(state, || {
            state.navigate_to(Route::Page1);
        });

        let navigate_to_page2 = bind!(state, || {
            state.navigate_to(Route::Page2);
        });

        let child = state.route.route.render_value(|value| {
            match value {
                Route::Page1 => dom! { <div>"Page 1"</div> },
                Route::Page2 => dom! { <div>"Page 2"</div> },
                Route::NotFound => dom! { <div>"Page Not Found"</div> },
            }
        });

        dom! {
            <div>
                <div>
                    "My Page"
                    <button on_click={navigate_to_page1}>"Page 1"</button>
                    <button on_click={navigate_to_page2}>"Page 2"</button>
                </div>
                {child}
            </div>
        }
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set(route);
    }
}
