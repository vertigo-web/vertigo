use serde::{Deserialize, Serialize};
use vertigo::{css, AutoMap, Css, LazyCache, SerdeRequest, Value, bind, get_driver, DomElement, dom, Resource, bind2};

#[derive(PartialEq, Eq, Clone)]
enum View {
    Main,
    Post { id: u32 },
    User { email: String },
}

#[derive(PartialEq, Eq, Serialize, Deserialize, SerdeRequest, Debug, Clone)]
struct PostModel {
    id: u32,
    title: String,
    body: String,
    // userId: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, SerdeRequest, Debug, Clone, PartialOrd)]
struct CommentModel {
    id: u32,
    body: String,
    email: String,
    name: String,
    // postId: u32,
}

#[derive(Clone)]
pub struct TodoState {
    view: Value<View>,
    // Resources
    posts: LazyCache<Vec<PostModel>>, //Vec<<>>
    comments: AutoMap<u32, LazyCache<Vec<CommentModel>>>,
}

impl TodoState {
    pub fn new() -> TodoState {
        let view = Value::new(View::Main);

        let posts = LazyCache::new(10 * 60 * 60 * 1000, move || async move {
            let request = get_driver().request("https://jsonplaceholder.typicode.com/posts").get();

            request.await.into(|status, body| {
                if status == 200 {
                    Some(body.into_vec::<PostModel>())
                } else {
                    None
                }
            })
        });

        let comments = AutoMap::new({
            move |post_id: &u32| -> LazyCache<Vec<CommentModel>> {
                let post_id = *post_id;

                LazyCache::new(10 * 60 * 60 * 1000, move || async move {
                    let request = get_driver()
                        .request(format!(
                            "https://jsonplaceholder.typicode.com/posts/{}/comments",
                            post_id
                        ))
                        .get();

                    request.await.into(|status, body| {
                        if status == 200 {
                            Some(body.into_vec::<CommentModel>())
                        } else {
                            None
                        }
                    })
                })
            }
        });

        TodoState {
            view,
            posts,
            comments,
        }
    }

    pub fn render(&self) -> DomElement {
        todo_render(self)
    }
}

fn todo_render(state: &TodoState) -> DomElement {
    let state = state.clone();

    let render = state.view.render_value({
        let state = state.clone();
        move |view|{
            match view {
                View::Main => todo_main_render(&state),
                View::Post { id } => todo_post_render(&state, id),
                View::User { email } => {
                    let view = state.view.clone();
                    let messag: String = format!("user = {}", email);

                    let on_click = move || {
                        view.set(View::Main);
                    };
                    
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

fn css_hover_item() -> Css {
    css! {"
        cursor: pointer;
        :hover {
            background-color: #e0e0e0;
        }
    "}
}

fn todo_main_render(state: &TodoState) -> DomElement {
    let state = state.clone();

    let posts = state.posts.to_computed().render_value(move |posts| {
        let todo_state = state.clone();

        match posts {
            Resource::Ready(posts) => {
                let result = dom! {
                    <div />
                };

                for post in posts.as_ref() {
                    let on_click = {
                        let view = todo_state.view.clone();
                        let id = post.id;

                        move || {
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
    
                result
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

fn css_comment_wrapper() -> Css {
    css! {"
        border: 1px solid black;
        padding: 5px;
        margin: 5px;
    "}
}

fn css_comment_author() -> Css {
    css! {"
        font-weight: bold;
        margin-right: 5px;
    "}
}

fn css_comment_body() -> Css {
    css! {"
    "}
}

fn todo_post_render(state: &TodoState, post_id: u32) -> DomElement {
    let state = state.clone();

    let view = state.view.clone();

    let on_click = bind(&view).call(|_, view| {
        view.set(View::Main);
    });

    let message = render_message(post_id);
    let comments_out = render_comments(&state, post_id);

    dom! {
        <div>
            { message }
            <div css={css_hover_item()} on_click={on_click}>
                "go to post list"
            </div>
            <hr />
            <hr />
            { comments_out }
        </div>
    }
}

fn render_message(post_id: u32) -> DomElement {
    let message = format!("post_id = {}", post_id);

    dom! {
        <div>
            { message }
        </div>
    }
}

fn render_comments(state: &TodoState, post_id: u32) -> DomElement {
    let view = state.view.clone();

    let comments = state.comments.get(&post_id);

    let comments_component = comments.to_computed().render_value(move |value| {
        let view = view.clone();
            
        match value {
            Resource::Ready(list) => {
                let result = dom! {
                    <div>
                        <div css={css_comment_wrapper()}>
                            <strong>
                                "Comments:"
                            </strong>
                        </div>
                    </div>
                };

                for comment in list.as_ref() {
                    let on_click_author = bind2(&view, &comment.email)
                        .call(|_, view, email| {
                            view.set(View::User { email: email.clone() });
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

                result
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
