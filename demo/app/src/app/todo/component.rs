use vertigo::{bind, css, dom, dom_element, Css, DomNode, Resource, Value};

use crate::app::todo::Select;

use super::state::{TodoState, View};

pub struct Todo { }

impl Todo {
    pub fn into_component(self) -> Self { self }

    pub fn mount(&self) -> DomNode {
        let state = TodoState::new();

        let render = state.view.render_value({
            let state = state.clone();
            move |view| -> DomNode {
                match view {
                    View::Main => todo_main_render(&state),
                    View::Post { id } => todo_post_render(&state, id),
                    View::User { email } => {
                        let messag: String = format!("user = {email}");

                        let view = &state.view;
                        let on_click = bind!(view, |_| {
                            view.set(View::Main);
                        });

                        dom!{
                            <div>
                                <div>
                                    { messag }
                                </div>
                                <div on_click={on_click} css={css_hover_item()}>
                                    "go to post list"
                                </div>
                            </div>
                        }
                    }
                }
            }
        });

        dom! {
            <div>
                { render }
            </div>
        }
    }
}

fn todo_main_render(state: &TodoState) -> DomNode {
    let state = state.clone();

    let posts = state.posts.to_computed().render_value(move |posts| -> DomNode {
        let todo_state = state.clone();

        match posts {
            Resource::Ready(posts) => {
                let result = dom_element! {
                    <div />
                };

                for post in posts.as_ref() {
                    let on_click = {
                        let view = todo_state.view.clone();
                        let id = post.id;

                        move |_| {
                            view.set(View::Post { id });
                        }
                    };

                    result.add_child(dom! {
                        <div on_click={on_click} css={css_hover_item()}>
                            "post = "
                            { post.title.clone() }
                        </div>
                    });
                }

                result.into()
            },
            Resource::Error(message) => {
                dom! {
                    <div>
                        "Error loading posts "
                        { message }
                    </div>
                }
            },
            Resource::Loading => {
                dom! {
                    <div>
                        "loading ..."
                        <vertigo-suspense />
                    </div>
                }
            }
        }
    });

    dom! {
        <div>
            {posts}
        </div>
    }
}

fn todo_post_render(state: &TodoState, post_id: u32) -> DomNode {
    let state = state.clone();

    let message = render_message(post_id);
    let comments_out = render_comments(&state, post_id);

    let view = state.view;

    let on_click = bind!(view, |_| {
        view.set(View::Main);
    });

    let authors = state.comments.get(&post_id)
        .to_computed()
        .map(|comments_res| {
            let mut options = vec!["".to_string()];
            if let Resource::Ready(comments) = comments_res {
                for comment in comments.iter() {
                    options.push(comment.email.clone());
                }
            }
            options
        });

    let selected_author = Value::default();

    dom! {
        <div>
            { message }
            <div css={css_hover_item()} on_click={on_click}>
                "go to post list"
            </div>
            <hr />
            "Select author: " <Select value={selected_author.clone()} options={authors} />
            "Selected author: " {selected_author}
            <hr />
            { comments_out }
        </div>
    }
}

fn render_message(post_id: u32) -> DomNode {
    let message = format!("post_id = {post_id}");

    dom! {
        <div>
            { message }
        </div>
    }
}

fn render_comments(state: &TodoState, post_id: u32) -> DomNode {
    let view = state.view.clone();

    let comments = state.comments.get(&post_id);

    let comments_component = comments.to_computed().render_value(move |value| {
        let view = view.clone();

        match value {
            Resource::Ready(list) => {
                let result = dom_element! {
                    <div>
                        <div css={css_comment_wrapper()}>
                            <strong>
                                "Comments:"
                            </strong>
                        </div>
                    </div>
                };

                for comment in list.as_ref() {
                    let on_click_author = bind!(view, comment, |_| {
                        view.set(View::User { email: comment.email.clone() });
                    });

                    let css_author = css_comment_author().extend(css_hover_item());

                    result.add_child(dom! {
                        <div css={css_comment_wrapper()}>
                            <span css={css_author} on_click={on_click_author}>
                                {&comment.email}
                            </span>
                            <span css={css_comment_body()}>
                                { &comment.body }
                            </span>
                        </div>
                    });
                }

                result.into()
            },
            Resource::Error(message) => {
                dom! {
                    <div>
                        "Error = "
                        { message }
                    </div>
                }
            },
            Resource::Loading => {
                dom! {
                    <div css={css_comment_wrapper()}>
                        <strong>
                            "Loading ..."
                            <vertigo-suspense />
                        </strong>
                    </div>
                }
            }
        }
    });

    dom! {
        <div>
            { comments_component }
        </div>
    }
}

fn css_hover_item() -> Css {
    css!("
        cursor: pointer;
        :hover {
            background-color: #e0e0e0;
        }
    ")
}

fn css_comment_wrapper() -> Css {
    css!("
        border: 1px solid black;
        padding: 5px;
        margin: 5px;
    ")
}

fn css_comment_author() -> Css {
    css!("
        font-weight: bold;
        margin-right: 5px;
    ")
}

fn css_comment_body() -> Css {
    css!("
    ")
}
