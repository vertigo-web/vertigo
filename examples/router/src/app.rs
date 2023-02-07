use vertigo::router::Router;
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
            "/page2" => Self::Page2,
            _ => Self::NotFound,
        }
    }
}

impl ToString for Route {
    fn to_string(&self) -> String {
        match self {
            Self::Page1 => "/page1",
            Self::Page2 => "/page2",
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
    pub route: Router<Route>,
}

impl App {
    pub fn new() -> Self {
        let route = Router::new_history_router();

        Self {
            route,
        }
    }

    pub fn render(&self) -> DomElement {
        let state = self;

        let navigate_to_page1 = bind!(state, || {
            state.navigate_to(Route::Page1);
        });

        let child = state.route.route.render_value(|value| {
            match value {
                Route::Page1 => dom! { <div>"Page 1"</div> },
                Route::Page2 => dom! { <div>"Page 2"</div> },
                Route::NotFound => dom! { <div>"Page Not Found"</div> },
            }
        });

        dom! {
            <html>
                <head />
                <body>
                    <div>
                        <div>
                            "My Page"
                            <button on_click={navigate_to_page1}>"Page 1"</button>
                            <a href={Route::Page2.to_string()}>"Page 2"</a>
                        </div>
                        {child}
                    </div>
                </body>
            </html>
        }
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set(route);
    }
}
