use vertigo::{bind, css, dom, transaction, Computed, DomNode, Resource};

use super::State;

pub struct GitHubExplorer {
    pub state: State,
}

impl GitHubExplorer {
    pub fn into_component(self) -> Self { self }

    pub fn mount(&self) -> DomNode {
        let state = &self.state;

        let on_input_callback = bind!(state, |new_value: String| {
            log::info!(" new value {new_value}");
            state.repo_input.set(new_value);
        });

        let on_show = bind!(state, |_| {
            transaction(|ctx| {
                let value = state.repo_input.get(ctx);
                log::info!(" new value {value}");
                state.repo_shown.set(value);
            });
        });

        let wrapper = css! {"
            border: 1px solid black;
            margin: 20px 0;
            padding: 10px;
        "};

        let input_css = css! {"
            margin-left: 10px;
        "};

        let button_css = css! {"
            margin: 0 10px;
            cursor: pointer;
        "};

        dom! {
            <div css={wrapper}>
                "Enter author/repo tuple: "
                <input css={input_css} value={state.repo_input.to_computed()} on_input={on_input_callback} />
                <button css={&button_css} on_click={on_show}>"Fetch"</button>
                <div css={button_css}>
                    <text computed={&state.repo_shown} />
                </div>
                { self.render_commit() }
            </div>
        }
    }

    fn render_commit(&self) -> DomNode {
        let commit_message = Computed::from({
            let state = self.state.clone();

            move |context| {
                let repo_shown = state.repo_shown.get(context);
                match repo_shown.as_str() {
                    "" => "".to_string(),
                    _ => match state.data.get(&repo_shown).get(context) {
                        Resource::Loading => "Loading...".to_string(),
                        Resource::Ready(branch) => branch.as_ref().commit.sha.clone(),
                        Resource::Error(err) => format!("Error: {err}"),
                    },
                }
            }
        });

        commit_message.render_value(|message| {
            let text_css = css! {"
                width: 600px;
                height: 300px;
                border: 1px solid black;
                padding: 5px;
                margin: 10px;
            "};

            dom! {
                <div css={text_css}>
                    { message }
                </div>
            }
        })
    }
}
