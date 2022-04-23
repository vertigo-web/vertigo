use vertigo::{css_fn, html, Resource, VDomElement, bind};

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

pub fn render(state: &State) -> VDomElement {
    let on_input_callback = bind(state).call_param(|state, new_value: String| {
        log::info!(" nowa wartosc3 {}", new_value);
        state.repo_input.set_value(new_value);
    });

    let on_show = bind(state).call(|state| {
        let value = state.repo_input.get_value().as_ref().clone();
        log::info!(" nowa wartosc3 {}", value);
        state.repo_shown.set_value(value);
    });

    let repo_input = state.repo_input.get_value();
    let repo_shown = state.repo_shown.get_value();

    let commit_sha = match repo_shown.as_str() {
        "" => "".to_string(),
        _ => match state.data.get_value(&repo_shown).get() {
            Resource::Loading => "Loading...".to_string(),
            Resource::Ready(branch) => branch.commit.sha,
            Resource::Error(err) => format!("Error: {}", err),
        },
    };

    html! {
        <div css={wrapper()}>
            "Enter author/repo tuple: "
            <input css={input_css()} value={(*repo_input).as_str()} on_input={on_input_callback} />
            <button css={button_css()} on_click={on_show}>"Fetch"</button>
            <div css={button_css()}>
                { repo_shown.as_str() }
            </div>
            <div css={text_css()}>
                { commit_sha }
            </div>
        </div>
    }
}
