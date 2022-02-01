use serde::{Deserialize, Serialize};
use std::rc::Rc;
use vertigo::{css, html, AutoMap, Css, Driver, LazyCache, Resource, SerdeRequest, VDomElement, Value, VDomComponent};

#[derive(PartialEq)]
enum View {
    Main,
    Post { id: u32 },
    User { email: String },
}

#[derive(PartialEq, Serialize, Deserialize, SerdeRequest, Debug)]
struct PostModel {
    id: u32,
    title: String,
    body: String,
    // userId: u32,
}

#[derive(PartialEq, Serialize, Deserialize, SerdeRequest, Debug)]
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
    comments: AutoMap<u32, Rc<LazyCache<Vec<CommentModel>>>>,
}

impl TodoState {
    pub fn component(driver: &Driver) -> VDomComponent {
        let view = driver.new_value(View::Main);

        let posts = LazyCache::new(driver, 10 * 60 * 60 * 1000, move |driver: Driver| {
            let request = driver.request("https://jsonplaceholder.typicode.com/posts").get();

            LazyCache::result(async move {
                request.await.into(|status, body| {
                    if status == 200 {
                        Some(body.into_vec::<PostModel>())
                    } else {
                        None
                    }
                })
            })
        });

        let comments = AutoMap::new({
            let driver = driver.clone();

            move |post_id: &u32| -> Rc<LazyCache<Vec<CommentModel>>> {
                let post_id = *post_id;

                Rc::new(LazyCache::new(&driver, 10 * 60 * 60 * 1000, move |driver: Driver| {
                    let request = driver
                        .request(format!(
                            "https://jsonplaceholder.typicode.com/posts/{}/comments",
                            post_id
                        ))
                        .get();

                    LazyCache::result(async move {
                        request.await.into(|status, body| {
                            if status == 200 {
                                Some(body.into_vec::<CommentModel>())
                            } else {
                                None
                            }
                        })
                    })
                }))
            }
        });

        let state = TodoState {
            view,
            posts,
            comments,
        };

        VDomComponent::new(state, todo_render)
    }
}

fn todo_render(state: &TodoState) -> VDomElement {
    match state.view.get_value().as_ref() {
        View::Main => {
            let main = TodoMainState::component(state);

            html! {
                <div>
                    { main }
                </div>
            }
        }
        View::Post { id } => {
            let post_view = TodoPostState::component(state, *id);

            html! {
                <div>
                    { post_view }
                </div>
            }
        }
        View::User { email } => {
            let view = state.view.clone();
            let messag = format!("user = {}", email);

            let on_click = move || {
                view.set_value(View::Main);
            };

            html! {
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

#[derive(Clone)]
struct TodoMainState {
    state: TodoState,
}

impl TodoMainState {
    fn component(state: &TodoState) -> VDomComponent {
        let state = TodoMainState { state: state.clone() };
        VDomComponent::new(state, todo_main_render)
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

fn todo_main_render(state_value: &TodoMainState) -> VDomElement {
    let todo_state = &state_value.state;

    let posts = todo_state.posts.get_value();

    match posts.as_ref() {
        Resource::Error(err) => {
            let message = format!("Error loading posts {}", err);
            html! {
                <div>
                    { message }
                </div>
            }
        }
        Resource::Loading => {
            html! {
                <div>
                    "loading ..."
                </div>
            }
        }
        Resource::Ready(list) => {
            let mut out: Vec<VDomElement> = Vec::new();

            for item in list {
                let message = format!("post = {}", item.title);

                let on_click = {
                    let view = todo_state.view.clone();
                    let id = item.id;

                    move || {
                        view.set_value(View::Post { id });
                    }
                };

                out.push(html! {
                    <div on_click={on_click} css={css_hover_item()}>
                        { message }
                    </div>
                });
            }

            html! {
                <div>
                    { ..out }
                </div>
            }
        }
    }
}

#[derive(Clone)]
struct TodoPostState {
    state: TodoState,
    post_id: u32,
}

impl TodoPostState {
    pub fn component(state: &TodoState, post_id: u32) -> VDomComponent {
        let state = TodoPostState {
            state: state.clone(),
            post_id
        };
        VDomComponent::new(state, todo_post_render)
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

fn todo_post_render(state_value: &TodoPostState) -> VDomElement {
    let post_id = state_value.post_id;
    let message = format!("post_id = {}", post_id);
    let view = state_value.state.view.clone();

    let on_click = {
        let view = view.clone();
        move || {
            view.set_value(View::Main);
        }
    };

    let comments = state_value.state.comments.get_value(&post_id);
    let comments_list = comments.get_value();

    let mut comments_out: Vec<VDomElement> = Vec::new();

    if let Resource::Ready(list) = comments_list.as_ref() {
        comments_out.push(html! {
            <div css={css_comment_wrapper()}>
                <strong>"Comments:"</strong>
            </div>
        });

        for comment in list.iter() {
            let on_click_author = {
                let view = view.clone();
                let email = comment.email.clone();
                move || {
                    view.set_value(View::User { email: email.clone() });
                }
            };

            let css_author = css_comment_author().extend(css_hover_item());

            comments_out.push(html! {
                <div css={css_comment_wrapper()}>
                    <span css={css_author} on_click={on_click_author}>
                        {&comment.email}
                    </span>
                    <span css={css_comment_body()}>
                        {&comment.body}
                    </span>
                </div>
            })
        }
    }

    if let Resource::Loading = comments_list.as_ref() {
        comments_out.push(html! {
            <div css={css_comment_wrapper()}>
                <strong>"Loading ..."</strong>
            </div>
        });
    }

    html! {
        <div>
            <div>
                { message }
            </div>
            <div on_click={on_click} css={css_hover_item()}>
                "go to post list"
            </div>
            <hr/>
            <hr/>

            { ..comments_out }
        </div>
    }
}
