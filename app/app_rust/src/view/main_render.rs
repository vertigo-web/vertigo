
use std::rc::Rc;
use std::collections::HashMap;
use virtualdom::{
    vdom::{
        models::{
            VDom::VDom,
        },
    },
};

use crate::app_state::AppState;

// use std::cell::RefCell;
// thread_local! {
//     static cssMap: RefCell<CssState> = RefCell::new(CssState::new());
// }

// struct CssState {
//     data: HashMap<&'static str, u32>,
//     counter: u32,
// }

// impl CssState {
//     fn new() -> CssState {
//         CssState {
//             data: HashMap::new(),
//             counter: 1,
//         }
//     }

//     fn getNextCounter(&mut self) -> u32 {
//         let current = self.counter;
//         self.counter += 1;
//         current
//     }

//     fn get(&mut self, id: &'static str) -> u32 {
//         let result = self.data.get(id);

//         if let Some(result) = result {
//             return *result;
//         }

//         let idNum = self.getNextCounter();
//         self.data.insert(id, idNum);
//         idNum
//     }
// }

// fn css(rr: &'static str) -> u32 {
//     let id = cssMap.with(|state| {
//         let mut cssState = state.borrow_mut();
//         let counter = cssState.get(rr);

//         counter
//     });

//     log::info!("css funkcja {} -> {}", rr, &id);    //rr.as_ptr() as u64);
    
//     id
//}

    // let story = "Once upon a time...";

    // let ptr = story.as_ptr();
    // let ptr = ptr as u64;
    // println!("aaa {} aaa", ptr);

    //"aaaa".into()

enum CssGroup {
    CssStatic {
        value: &'static str,                    //&str -- moze zachowywac sie jako id po ktorym odnajdujemy interesujaca regule
    },
    CssDynamic {
        value: String,                          //w tym wypadku String, jest kluczem do hashmapy z wynikowa nazwa klasy       
    }
}

struct Css {
    groups: Vec<CssGroup>,
}

impl Css {
    pub fn new() -> Css {
        Css {
            groups: Vec::new()
        }
    }

    pub fn add(mut self, value: &'static str) -> Self {
        self.groups.push(CssGroup::CssStatic {
            value
        });
        self
    }

    pub fn str(&mut self, value: &'static str) {
        self.groups.push(CssGroup::CssStatic {
            value
        })
    }

    pub fn dynamic(&mut self, value: String) {
        self.groups.push(CssGroup::CssDynamic {
            value
        })
    }
}

fn wrapper1() -> Css {
    Css::new().add("windth: 30px; height: 20px;")
}

fn wrapper2(active: bool) -> Css {
    let mut out = Css::new().add("windth: 30px; height: 20px;");

    if active {
        out.str("color: red;");
    }

    let url: Option<String> = None;
    if let Some(url) = url {
        
    }

    out
}

//"border: 1px solid black; padding: 10px; background-color: #e0e0e0;")

/*
    kady statyczny string jest zapisany tylko raz.
    więc kademu statycznemu stringowi będzie odpowiadał jakiś identyfikator
*/

    // wrapper1();
    // wrapper2(true);
    // wrapper2(false);

pub fn main_render(app_state: &Rc<AppState>) -> Vec<VDom> {
    use virtualdom::vdom::models::{node, text};

    let app_state = app_state.clone();

    let onDown = {
        let app_state = app_state.clone();
        move || {
            app_state.decrement();
        }
    };

    let onUp = {
        let app_state = app_state.clone();
        move || {
            log::info!("on click");
            app_state.increment();
        }
    };

    let at = app_state.at.getValue();
    let value = app_state.value.getValue();

    vec!(
        node("div")
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("bla bla bla"))
            )
            .child(node("div")
                //.style(wrapper2(true))                //TODO - zaimplementować
                //.style(wrapper1())                    //TODO - zaimplementować
                .onClick(onUp.clone())
                .child(
                    text(format!("aktualna wartosc = {} ({})", value, at))
                )
            )
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("up"))
                .onClick(onUp)
            )
            .child(node("div")
                .attr("style", "border: 1px solid black; padding: 10px; background-color: #e0e0e0;")
                .child(text("down"))
                .onClick(onDown)
            )
            .child(node("div")
                .child(text(format!("jakis footer {} {}", *value % 2, *value % 3)))
            )
    )

    /*
    <div aaa="one" bbb="two">
        "Abudabi"
    </div>
    */
}
