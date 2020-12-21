use virtualdom::{DomDriver, FetchMethod, computed::{
        Dependencies,
        Value,
        Computed,
    }};

use crate::simple_counter;
use crate::sudoku;

pub struct State {
    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: Computed<simple_counter::State>,
    pub counter2: Computed<simple_counter::State>,
    pub counter3: Computed<simple_counter::State>,
    pub counter4: Computed<simple_counter::State>,

    pub suma: Computed<u32>,

    pub sudoku: Computed<sudoku::Sudoku>,
}

impl State {
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {
        let counter1 = simple_counter::State::new(&root);
        let counter2 = simple_counter::State::new(&root);
        let counter3 = simple_counter::State::new(&root);
        let counter4 = simple_counter::State::new(&root);

        let suma = {
            let counter1 = counter1.clone();
            let counter2 = counter2.clone();
            let counter3 = counter3.clone();
            let counter4 = counter4.clone();

            root.from(move || {
                let value1 = *counter1.getValue().counter.getValue();
                let value2 = *counter2.getValue().counter.getValue();
                let value3 = *counter3.getValue().counter.getValue();
                let value4 = *counter4.getValue().counter.getValue();

                value1 + value2 + value3 + value4
            })
        };

        // run1(&driver);
        // run2(&driver);
        run3(&driver);

        root.newComputedFrom(State {
            value: root.newValue(33),
            at: root.newValue(999),
            counter1,
            counter2,
            counter3,
            counter4,
            suma,
            sudoku: sudoku::Sudoku::new(root)
        })
    }

    pub fn increment(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr - 1);
    }

    // async fn cos() -> u32 {
    //     4
    // }
}


// fn run1(driver: &DomDriver) {
//     let driver_span = driver.clone();
//     driver.spawn_local(async move {
//         let url: String = "https://api.github.com/feeds".into();
//         let response = driver_span.fetch(FetchMethod::GET, url, None, None).await;
//         log::info!("Odpowiedź {}", response);
//     });
// }

// fn run2(driver: &DomDriver) {
//     let driver_span2 = driver.clone();

//     driver.spawn_local(async move {
//         let url: String = "http://127.0.0.1:4000/api/list.json".into();
//         let response = driver_span2.fetch(FetchMethod::GET, url, None, None).await;
//         log::info!("Odpowiedź z listą {}", response);
//     });
// }

fn run3(driver: &DomDriver) {
    let driver_span = driver.clone();

    driver.spawn_local(async move {
        use serde::{Deserialize, Serialize};
        
        #[derive(Debug, Serialize, Deserialize)]
        pub struct Branch {
            pub name: String,
            pub commit: Commit,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Commit {
            pub sha: String,
            pub commit: CommitDetails,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CommitDetails {
            pub author: Signature,
            pub committer: Signature,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Signature {
            pub name: String,
            pub email: String,
        }

        let repo: String = "rustwasm/wasm-bindgen".into();

        let url = format!("https://api.github.com/repos/{}/branches/master", repo);

        let response = driver_span.fetch(FetchMethod::GET, url, None, None).await;

        match response {
            Ok(response) => {
                let branch = serde_json::from_str::<Branch>(response.as_str()).unwrap();

                log::info!("odpowiedź z serwera {:?}", branch);
            },
            Err(_) => {
                log::error!("Error fetch branch");
            }
        }
    });
}