use vertigo::{css_fn, Resource, bind, DomElement, dom, DomComment, Computed};

use super::State;

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

fn render_commit(state: &State) -> DomComment {
    let commit_message = Computed::from({
        let state = state.clone();

        move |context| {
            let repo_shown = state.repo_shown.get(context);
            match repo_shown.as_str() {
                "" => "".to_string(),
                _ => match state.data.get(&repo_shown).get(context) {
                    Resource::Loading => "Loading...".to_string(),
                    Resource::Ready(branch) => branch.as_ref().commit.sha.clone(),
                    Resource::Error(err) => format!("Error: {}", err),
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

pub fn render(state: &State) -> DomElement {
    let on_input_callback = bind(state).call_param(|_, state, new_value: String| {
        log::info!(" nowa wartosc3 {}", new_value);
        state.repo_input.set(new_value);
    });

    let on_show = bind(state).call(|context, state| {
        let value = state.repo_input.get(context);
        log::info!(" nowa wartosc3 {}", value);
        state.repo_shown.set(value);
    });

    dom! {
        <div css={wrapper()}>
            "Enter author/repo tuple: "
            <input css={input_css()} value={state.repo_input.to_computed()} on_input={on_input_callback} />
            <button css={button_css()} on_click={on_show}>"Fetch"</button>
            <div css={button_css()}>
                <text computed={&state.repo_shown} />
            </div>
            { render_commit(state) }
        </div>
    }
}
