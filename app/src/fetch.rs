use browserdriver::DomDriverBrowser;
use serde::{Deserialize, Serialize};

use virtualdom::FetchMethod;
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


pub async fn run(repo: String) -> Branch {

    let driver = DomDriverBrowser::new();

    let url = format!("https://api.github.com/repos/{}/branches/master", repo);

    let response = driver.fetch(FetchMethod::GET, url, None, None).await;

    let branch_info = serde_json::from_str::<Branch>(response.as_str()).unwrap();

    branch_info
}
