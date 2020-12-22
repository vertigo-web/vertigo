use virtualdom::{
    computed::Computed,
    VDomNode,
    NodeAttr,
    Css,
};

use super::state::{State, Resource};

fn wrapper() -> Css {
    Css::one("
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    ")
}

fn input_css() -> Css {
    Css::one("
        margin-left: 10px;
    ")
}

fn button_css() -> Css {
    Css::one("
        margin: 0 10px;
        cursor: pointer;
    ")
}

fn text_css() -> Css {
    Css::one("
        width: 600px;
        height: 300px;
        border: 1px solid black;
        padding: 5px;
        margin: 10px;
    ")
}

pub fn render(state: &Computed<State>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, attr, onClick, onInput};

    let state = state.getValue();

    let on_input = {
        let value = state.repo_input.clone();
        move |new_value: String| {
            log::info!(" nowa wartosc3 {}", new_value);
            value.setValue(new_value);
        }
    };

    let on_show = {
        let value = state.repo_input.getValue();
        let state_inner = state.clone();
        move || {
            log::info!(" nowa wartosc3 {}", value);
            state_inner.repo_shown.setValue((*value).clone());
        }
    };

    let repo_input = state.repo_input.getValue();
    let repo_shown = state.repo_shown.getValue();

    let commit_sha = match repo_shown.as_str() {
        "" => "".to_string(),
        _ => match &*state.data.getValue(&repo_shown).getValue() {
            Resource::Loading => "Loading...".to_string(),
            Resource::Ready(branch) => branch.commit.sha.clone(),
            Resource::Failed(err) => format!("Error: {}", err),
        }
    };

    buildNode("div", vec!(
        css(wrapper()),
        text("Enter author/repo tuple:"),
        node("input", vec!(
            css(input_css()),
            attr("value", (*repo_input).as_str()),
            onInput(on_input),
        )),
        node("button", vec!(
            css(button_css()),
            onClick(on_show),
            text("Fetch")
        )),
        node("div", vec!(
            css(button_css()),
            text(repo_shown.as_str())
        )),
        node("div", vec!(
            css(text_css()),
            text(commit_sha)
        ))
    ))
}