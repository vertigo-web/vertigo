use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use vertigo::{AutoMap, Computed, Driver, Resource, SerdeSingleRequest, Value, VDomComponent};

mod render;

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

#[derive(Debug, Serialize, Deserialize, SerdeSingleRequest, PartialEq, Clone)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

fn fetch_repo(repo: &str, value: Value<Resource<Branch>>, driver: &Driver) {
    driver.spawn({
        let driver = driver.clone();
        let url = format!("https://api.github.com/repos/{}/branches/master", repo);
        log::info!("Fetching {}", url);

        async move {
            let response = driver.request(url).get().await.into(|status, body| {
                if status == 200 {
                    return Some(body.into::<Branch>());
                }

                None
            });

            value.set_value(response);
        }
    });
}

#[derive(PartialEq, Clone)]
pub struct Item {
    value: Computed<Resource<Branch>>,
}

impl Item {
    pub fn new(driver: &Driver, repo_name: &str) -> Item {
        log::info!("Creating for {}", repo_name);
        let new_value = driver.new_value(Resource::Loading);
        let new_computed = new_value.to_computed();

        fetch_repo(repo_name, new_value, driver);

        Item { value: new_computed }
    }

    pub fn get(&self) -> Resource<Branch> {
        self.value.get_value().ref_clone()
    }
}

#[derive(PartialEq)]
pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Item>,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let driver = driver.clone();

        let state = State {
            repo_input: driver.new_value(String::from("")),
            repo_shown: driver.new_value(String::from("")),
            data: AutoMap::new({
                let driver = driver.clone();

                move |repo_name: &String| Item::new(&driver, repo_name)
            }),
        };

        driver.bind_render(state, render::render)
    }
}

