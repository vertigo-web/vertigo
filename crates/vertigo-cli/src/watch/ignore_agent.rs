use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    Match,
};
use std::path::PathBuf;

use super::WatchOpts;

pub struct IgnoreAgents(Vec<Gitignore>);

impl IgnoreAgents {
    pub fn new(root: &PathBuf, opts: &WatchOpts) -> Self {
        // Generate one Gitignore instance per every watched directory
        let mut ignore_agents = if let Some(agent) = create_ignore_agent(root, opts) {
            vec![agent]
        } else {
            vec![]
        };
        for watch_dir in &opts.add_watch_path {
            if let Some(agent) = create_ignore_agent(&PathBuf::from(watch_dir), opts) {
                ignore_agents.push(agent);
            }
        }

        IgnoreAgents(ignore_agents)
    }

    pub fn should_be_ignored(&self, path: &PathBuf) -> bool {
        for agent in &self.0 {
            match agent.matched(path, path.is_dir()) {
                Match::None => (),
                Match::Ignore(_glob) => return true,
                Match::Whitelist(_glob) => (),
            }
        }
        false
    }
}

fn create_ignore_agent(root: &PathBuf, opts: &WatchOpts) -> Option<Gitignore> {
    log::debug!("Building ignore agent for root {}", root.to_string_lossy());

    let mut gitignore = GitignoreBuilder::new(root);

    for ignore_list in opts.watch_ignore_lists.split_ascii_whitespace() {
        let path = root.join(ignore_list);
        log::debug!("- adding ignore list from {}", path.to_string_lossy());
        let _ = gitignore.add(path);
    }

    log::debug!("- adding global ignores: {}", opts.global_ignores);

    for glob in opts.global_ignores.split_ascii_whitespace() {
        let _ = gitignore.add_line(Some(root.clone()), glob);
    }

    gitignore
        .build()
        .inspect_err(|_| {
            log::error!(
                "Error building gitignore parser for root {}",
                root.to_string_lossy()
            )
        })
        .ok()
}
