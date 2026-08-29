use vertigo::{Computed, DomNode, Resource, css, dom, transaction};

use crate::app::github_explorer::state::{
    Branch, Signature, state_github_branch_name, state_github_repo_input, state_github_repo_shown,
};

pub struct GitHubExplorer {}

impl GitHubExplorer {
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let on_input_callback = |new_value: String| {
            log::info!(" new value {new_value}");
            state_github_repo_input().set(new_value);
        };

        let on_show = |_| {
            transaction(|ctx| {
                let value = state_github_repo_input().get(ctx);
                log::info!(" new value {value}");
                state_github_repo_shown().set(value);
            });
        };

        let wrapper = css! {"
            border: 1px solid black;
            margin: 20px 0;
            padding: 10px;
        "};

        let input_css = css! {"
            margin-left: 10px;
            border: black 1px solid;
        "};

        let button_css = css! {"
            margin: 0 10px;
            cursor: pointer;
        "};

        let shown_css = css! {"
            margin: 10px 0 0 10px;
        "};

        dom! {
            <div css={wrapper}>
                "Enter author/repo tuple: "
                <input css={input_css} value={state_github_repo_input().to_computed()} on_input={on_input_callback} />
                <button css={button_css} on_click={on_show}>"Fetch"</button>
                <div css={shown_css}>
                    "Showing: " { state_github_repo_shown() }
                </div>
                { self.render_commit() }
            </div>
        }
    }

    fn render_commit(&self) -> DomNode {
        let branch = Computed::from(move |context| {
            let repo_shown = state_github_repo_shown().get(context);

            match repo_shown.as_str() {
                "" => None,
                _ => Some(state_github_branch_name(&repo_shown).get(context)),
            }
        });

        branch.render_value(|branch| {
            let body = match branch {
                None => dom! { <div>"Nothing fetched yet."</div> },
                Some(Resource::Loading) => dom! { <div>"Loading..."</div> },
                Some(Resource::Error(err)) => dom! { <div>"Error: " { err }</div> },
                Some(Resource::Ready(branch)) => render_branch(&branch),
            };

            // A fixed width, so the box does not resize as it moves between these states.
            let text_css = css! {"
                width: 600px;
                min-height: 150px;
                border: 1px solid black;
                padding: 5px;
                margin: 10px;
            "};

            dom! {
                <div css={text_css}>
                    { body }
                </div>
            }
        })
    }
}

/// Everything the response carried, rather than just the sha.
///
/// `Branch` nests three deep - `commit.commit.author.name` - so rendering all of it is also
/// what shows that `AutoJsJson` decoded the whole tree and not only its top level.
fn render_branch(branch: &Branch) -> DomNode {
    let details = &branch.commit.commit;

    let name = branch.name.clone();
    let sha = branch.commit.sha.clone();

    dom! {
        <div>
            <div>"Branch: " { name }</div>
            <div>"Commit: " { sha }</div>
            { render_signature("Author", &details.author) }
            { render_signature("Committer", &details.committer) }
        </div>
    }
}

fn render_signature(role: &'static str, who: &Signature) -> DomNode {
    // Built in Rust rather than interpolated into the markup: the angle brackets around an
    // address are text, and spelling them as literals inside `dom!` reads like tags.
    let line = format!("{role}: {} <{}>", who.name, who.email);

    dom! {
        <div>{ line }</div>
    }
}
