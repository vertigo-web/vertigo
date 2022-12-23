use vertigo::{AutoMap, LazyCache, Value, RequestBuilder};
use vertigo::AutoJsJson;

#[derive(PartialEq, Eq, Clone)]
pub enum View {
    Main,
    Post { id: u32 },
    User { email: String },
}

#[derive(PartialEq, Eq, AutoJsJson, Debug, Clone)]
pub struct PostModel {
    pub id: u32,
    pub title: String,
    pub body: String,
    // pub userId: u32,
}

#[derive(PartialEq, Eq, AutoJsJson, Debug, Clone, PartialOrd)]
pub struct CommentModel {
    pub id: u32,
    pub body: String,
    pub email: String,
    pub name: String,
    // pub postId: u32,
}

#[derive(Clone)]
pub struct TodoState {
    pub view: Value<View>,
    pub posts: LazyCache<Vec<PostModel>>,
    pub comments: AutoMap<u32, LazyCache<Vec<CommentModel>>>,
}

impl TodoState {
    pub fn new() -> TodoState {
        let view = Value::new(View::Main);

        let posts = RequestBuilder::get("https://jsonplaceholder.typicode.com/posts")
            .ttl_minutes(10)
            .lazy_cache(|status, body| {
                if status == 200 {
                    Some(body.into::<Vec<PostModel>>())
                } else {
                    None
                }
            });

        let comments = AutoMap::new({
            move |post_id: &u32| -> LazyCache<Vec<CommentModel>> {
                RequestBuilder::get(format!("https://jsonplaceholder.typicode.com/posts/{post_id}/comments"))
                    .ttl_minutes(10)
                    .lazy_cache(|status, body| {
                        if status == 200 {
                            Some(body.into::<Vec<CommentModel>>())
                        } else {
                            None
                        }
                    })
            }
        });

        TodoState {
            view,
            posts,
            comments,
        }
    }
}
