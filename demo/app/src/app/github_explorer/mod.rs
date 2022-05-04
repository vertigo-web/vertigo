use serde::{Deserialize, Serialize};
use std::{cmp::PartialEq, rc::Rc};
use vertigo::{AutoMap, Resource, SerdeSingleRequest, Value, VDomComponent, LazyCache, get_driver};

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
    pub fn new(repo_name: &str) -> Item {
        log::info!("Creating for {}", repo_name);

        let url = format!("https://api.github.com/repos/{}/branches/master", repo_name);

        let branch = LazyCache::new(10 * 60 * 60 * 1000, move || {
            let url = url.clone();

            async move {
                let aa = get_driver().request(url).get().await.into(|status, body| {
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

    pub fn get(&self) -> Resource<Rc<Branch>> {
        self.branch.get()
    }
}

#[derive(Clone)]
pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Item>,
}

impl State {
    pub fn component() -> VDomComponent {
        let state = State {
            repo_input: Value::new(String::from("")),
            repo_shown: Value::new(String::from("")),
            data: AutoMap::new(move |repo_name: &String| Item::new(repo_name)),
        };

        VDomComponent::from(state, render::render)
    }
}

