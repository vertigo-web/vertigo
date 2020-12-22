use serde::{Deserialize, Serialize};

use virtualdom::{
    DomDriver,
    FetchMethod,
    computed::{
        Dependencies,
        Value,
        AutoMap,
        Computed,
    }
};

pub enum Resource<T> {
    Loading,
    Ready(T),
    Failed(String),
}

pub struct State {
    pub repo_input: Value<String>,
    pub repo_shown: Value<String>,
    pub data: AutoMap<String, Resource<Branch>>,
}

impl State {
    pub fn new(root: &Dependencies, driver: &DomDriver) -> Computed<State> {
        let root_inner = root.clone();
        let driver_inner = driver.clone();

        root.newComputedFrom(State {
            repo_input: root.newValue("".into()),
            repo_shown: root.newValue("".into()),
            data: AutoMap::new(root, move |repo_name: &String| -> Computed<Resource<Branch>> {
                log::info!("Creating for {}", repo_name);
                let new_value = root_inner.newValue(Resource::Loading);
                let new_computed = new_value.toComputed();
    
                fetch_repo(repo_name.as_str(), new_value, &driver_inner);
    
                new_computed
            }),
        })
    }
}


fn fetch_repo(repo: &str, value: Value<Resource<Branch>>, driver: &DomDriver) {
    let driver_span = driver.clone();
    let url = format!("https://api.github.com/repos/{}/branches/master", repo);
    log::info!("Fetching1 {}", url);

    driver.spawn_local(async move {
        log::info!("Fetching2 {}", url);
        let response = driver_span.fetch(FetchMethod::GET, url, None, None).await;

        match response {
            Ok(response) => {
                match serde_json::from_str::<Branch>(response.as_str()) {
                    Ok(branch) => {
                        log::info!("odpowiedź z serwera {:?}", branch);
                        value.setValue(Resource::Ready(branch));
                    },
                    Err(err) => {
                        log::error!("Error parsing response: {}", err);
                        value.setValue(Resource::Failed(err.to_string()));
                    }
                }
            },
            Err(_) => {
                log::error!("Error fetch branch")
            }
        }
    });
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

