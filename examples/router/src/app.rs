use vertigo::router::HashRouter;
use vertigo::{html, Computed, Driver, VDomElement, Value, VDomComponent};

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq)]
pub struct State {
    pub route: Value<Route>,

    hash_router: HashRouter,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let route: Value<Route> = driver.new_value(Route::new(&driver.get_hash_location()));

        let hash_router = HashRouter::new(driver, route.clone(), {
            let route = route.clone();

            Box::new(move |url: &String| {
                route.set_value(Route::new(url));
            })
        });

        let state = State {
            route,
            hash_router,
        };

        driver.bind_render(state, render)
    }

    pub fn navigate_to(&self, route: Route) {
        self.route.set_value(route);
    }
}

fn render(app_state: &Computed<State>) -> VDomElement {
    let state = app_state.get_value();

    let navigate_to_page1 = {
        let state = state.clone();
        move || {
            state.navigate_to(Route::Page1);
        }
    };

    let navigate_to_page2 = {
        let state = state.clone();
        move || {
            state.navigate_to(Route::Page2);
        }
    };

    let child = match *state.route.get_value() {
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
