use vertigo::{LazyCache, RequestBuilder, Value, store};
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

#[store]
pub fn state_todo_view() -> Value<View> {
    Value::new(View::Main)
}

#[store]
pub fn state_todo_posts() -> LazyCache<Vec<PostModel>> {
    RequestBuilder::get("https://jsonplaceholder.typicode.com/posts")
        .ttl_minutes(10)
        .lazy_cache(|status, body| {
            if status == 200 {
                Some(body.into::<Vec<PostModel>>())
            } else {
                None
            }
        })
}

#[store]
pub fn state_todo_comments(post_id: u32) -> LazyCache<Vec<CommentModel>> {
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
