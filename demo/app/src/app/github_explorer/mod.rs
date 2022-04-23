use serde::{Deserialize, Serialize};
use std::{cmp::PartialEq};
use vertigo::{AutoMap, Driver, Resource, SerdeSingleRequest, Value, VDomComponent, LazyCache};

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

#[derive(Clone)]
pub struct Item {
    branch: LazyCache<Branch>,
}

impl Item {
    pub fn new(driver: &Driver, repo_name: &str) -> Item {
        log::info!("Creating for {}", repo_name);

        let url = format!("https://api.github.com/repos/{}/branches/master", repo_name);

        let branch = LazyCache::new(driver, 10 * 60 * 60 * 1000, move |driver: Driver| {
            let url = url.clone();

            async move {
                let url = url.clone();
                let aa = driver.request(url).get().await.into(|status, body| {
                    if status == 200 {
                        return Some(body.into::<Branch>());
                    }

                    None
                });

                aa
            }
        });

        Item { branch }
    }

    pub fn get(&self) -> Resource<Branch> {
        self.branch.get_value().ref_clone()
    }
}

#[derive(Clone)]
pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Item>,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let state = State {
            repo_input: driver.new_value(String::from("")),
            repo_shown: driver.new_value(String::from("")),
            data: AutoMap::new({
                let driver = driver.clone();

                move |repo_name: &String| Item::new(&driver, repo_name)
            }),
        };

        VDomComponent::from(state, render::render)
    }
}

