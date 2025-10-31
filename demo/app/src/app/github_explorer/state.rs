use std::{cmp::PartialEq};
use vertigo::{LazyCache, RequestBuilder, Value, store};
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

#[store]
pub fn state_github_branch_name(repo_name: &String) -> LazyCache<Branch> {
    log::info!("Creating for {repo_name}");

    let url = format!("https://api.github.com/repos/{repo_name}/branches/master");
    RequestBuilder::get(url)
        .ttl_minutes(10)
        .lazy_cache(|status, body| {
            if status == 200 {
                return Some(body.into::<Branch>());
            }

            None
        })
}

#[store]
pub fn state_github_repo_input() -> Value<String> {
    Value::default()
}

#[store]
pub fn state_github_repo_shown() -> Value<String> {
    Value::default()
}


