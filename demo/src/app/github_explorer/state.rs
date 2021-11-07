use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};

use vertigo::{
    DomDriver,
    RequestTrait,
    Resource,
    computed::{
        Dependencies,
        Value,
        AutoMap,
        Computed,
    }};

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

impl RequestTrait for Branch {
    fn into_string(self) -> Result<String, String> {
        serde_json::to_string(&self)
            .map_err(|err| format!("error serialize {}", err))
    }

    fn from_string(data: &str) -> Result<Self, String> {
        serde_json::from_str::<Self>(data)
            .map_err(|err| format!("error deserialize {}", err))
    }
}

fn fetch_repo(repo: &str, value: Value<Resource<Branch>>, driver: &DomDriver) {
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
    pub fn new(root: &Dependencies, driver: &DomDriver, repo_name: &str) -> Item {
        log::info!("Creating for {}", repo_name);
        let new_value = root.new_value(Resource::Loading);
        let new_computed = new_value.to_computed();

        fetch_repo(repo_name, new_value, driver);

        Item {
            value: new_computed,
        }
    }

    pub fn get(&self) -> Resource<Branch>{
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
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {
        let root = root.clone();
        let driver = driver.clone();

        root.new_computed_from(State {
            repo_input: root.new_value(String::from("")),
            repo_shown: root.new_value(String::from("")),
            data: AutoMap::new({
                let root = root.clone();

                move |repo_name: &String| Item::new(&root, &driver, repo_name)
            })
        })
    }
}
