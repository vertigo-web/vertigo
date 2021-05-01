use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};

use vertigo::{
    DomDriver,
    computed::{
        Dependencies,
        Value,
        AutoMap,
        Computed,
    }
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CommitDetails {
    pub author: Signature,
    pub committer: Signature,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(PartialEq, Clone)]
pub enum Resource<T: PartialEq> {
    Loading,
    Ready(T),
    Failed(String),
}

fn fetch_repo(repo: &str, value: Value<Resource<Branch>>, driver: &DomDriver) {
    let driver_span = driver.clone();
    let url = format!("https://api.github.com/repos/{}/branches/master", repo);
    log::info!("Fetching1 {}", url);

    driver.spawn_local(async move {
        log::info!("Fetching2 {}", url);
        let response = driver_span.fetch(url).get().await;

        match response {
            Ok(response) => {
                match serde_json::from_str::<Branch>(response.as_str()) {
                    Ok(branch) => {
                        log::info!("odpowiedź z serwera {:?}", branch);
                        value.set_value(Resource::Ready(branch));
                    },
                    Err(err) => {
                        log::error!("Error parsing response: {}", err);
                        value.set_value(Resource::Failed(err.to_string()));
                    }
                }
            },
            Err(_) => {
                log::error!("Error fetch branch")
            }
        }
    });
}


#[derive(PartialEq, Clone)]
pub struct Item {
    value: Computed<Resource<Branch>>,
}

impl Item {
    pub fn new(root: &Dependencies, driver: &DomDriver, repo_name: &String) -> Item {
        log::info!("Creating for {}", repo_name);
        let new_value = root.new_value(Resource::Loading);
        let new_computed = new_value.to_computed();

        fetch_repo(repo_name, new_value, &driver);

        Item {
            value: new_computed,
        }
    }

    pub fn get(&self) -> Resource<Branch>{
        self.value.get_value().as_ref().clone()
    }
}

#[derive(PartialEq)]
pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Item>,
}

impl State {
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {
        let root = root.clone();
        let driver = driver.clone();

        root.new_computed_from(State {
            repo_input: root.new_value("".into()),
            repo_shown: root.new_value("".into()),
            data: AutoMap::new({
                let root = root.clone();

                move |repo_name: &String| Item::new(&root, &driver, repo_name)
            })
        })
    }
}
