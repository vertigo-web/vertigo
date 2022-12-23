use std::{cmp::PartialEq, rc::Rc};
use vertigo::{AutoMap, Resource, Value, LazyCache, Context, RequestBuilder};
use vertigo::AutoJsJson;

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone, Default)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone, Default)]
pub struct CommitDetails {
    pub author: Signature,
    pub committer: Signature,
}

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone, Default)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone, Default)]
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

        let url = format!("https://api.github.com/repos/{repo_name}/branches/master");
        let branch = RequestBuilder::get(url)
            .ttl_minutes(10)
            .lazy_cache(|status, body| {
                if status == 200 {
                    return Some(body.into::<Branch>());
                }

                None
            });


        Item { branch }
    }

    pub fn get(&self, context: &Context) -> Resource<Rc<Branch>> {
        self.branch.get(context)
    }
}

#[derive(Clone)]
pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Item>,
}

impl State {
    pub fn new() -> State {
        State {
            repo_input: Value::default(),
            repo_shown: Value::default(),
            data: AutoMap::new(move |repo_name: &String| Item::new(repo_name)),
        }
    }
}
