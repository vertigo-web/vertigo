use vertigo::router::HashRouter;
use vertigo::{html, VDomElement, VDomComponent, bind};

#[derive(Clone, PartialEq, Debug)]
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
pub struct State {
    pub route: HashRouter<Route>,
}

impl State {
    pub fn component() -> VDomComponent {
        let route = HashRouter::new();

        let state = State {
            route,
        };

        VDomComponent::from(state, render)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set(route);
    }
}

fn render(state: &State) -> VDomElement {
    let navigate_to_page1 = bind(state).call(|state| {
        state.navigate_to(Route::Page1);
    });

    let navigate_to_page2 = bind(state).call(|state| {
        state.navigate_to(Route::Page2);
    });

    let child = match state.route.get() {
        Route::Page1 => html! { <div>"Page 1"</div> },
        Route::Page2 => html! { <div>"Page 2"</div> },
        Route::NotFound => html! { <div>"Page Not Found"</div> },
    };

    html! {
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
