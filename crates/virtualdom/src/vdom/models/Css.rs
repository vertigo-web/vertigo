
enum CssGroup {
    CssStatic {
        value: &'static str,                    //&str -- moze zachowywac sie jako id po ktorym odnajdujemy interesujaca regule
    },
    CssDynamic {
        value: String,                          //w tym wypadku String, jest kluczem do hashmapy z wynikowa nazwa klasy       
    }
}

pub struct Css {
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
