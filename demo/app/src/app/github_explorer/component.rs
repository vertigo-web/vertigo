use vertigo::{css_fn, Resource, bind, DomElement, dom, Computed, DomCommentCreate, transaction};

use super::State;

pub struct GitHubExplorer {
    pub state: State
}

impl GitHubExplorer {
    pub fn mount(&self) -> DomElement {
        let state = &self.state;

        let on_input_callback = bind!(state, |new_value: String| {
            log::info!(" new value {}", new_value);
            state.repo_input.set(new_value);
        });

        let on_show = bind!(state, || {
            transaction(|ctx| {
                let value = state.repo_input.get(ctx);
                log::info!(" new value {}", value);
                state.repo_shown.set(value);
            });
        });

        dom! {
            <div css={wrapper()}>
                "Enter author/repo tuple: "
                <input css={input_css()} value={state.repo_input.to_computed()} on_input={on_input_callback} />
                <button css={button_css()} on_click={on_show}>"Fetch"</button>
                <div css={button_css()}>
                    <text computed={&state.repo_shown} />
                </div>
                { self.render_commit() }
            </div>
        }
    }

    fn render_commit(&self) -> DomCommentCreate {
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
            dom! {
                <div css={text_css()}>
                    { message }
                </div>
            }
        })
    }
}

css_fn! { wrapper, "
    border: 1px solid black;
    margin: 20px 0;
    padding: 10px;
" }

css_fn! { input_css, "
    margin-left: 10px;
" }

css_fn! { button_css, "
    margin: 0 10px;
    cursor: pointer;
" }

css_fn! { text_css, "
    width: 600px;
    height: 300px;
    border: 1px solid black;
    padding: 5px;
    margin: 10px;
" }
