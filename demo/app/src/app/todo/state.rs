use serde::{Deserialize, Serialize};
use vertigo::{AutoMap, LazyCache, SerdeRequest, Value, get_driver, bind};

#[derive(PartialEq, Eq, Clone)]
pub enum View {
    Main,
    Post { id: u32 },
    User { email: String },
}

#[derive(PartialEq, Eq, Serialize, Deserialize, SerdeRequest, Debug, Clone)]
pub struct PostModel {
    pub id: u32,
    pub title: String,
    pub body: String,
    // pub userId: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, SerdeRequest, Debug, Clone, PartialOrd)]
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
    // Resources
    pub posts: LazyCache<Vec<PostModel>>, //Vec<<>>
    pub comments: AutoMap<u32, LazyCache<Vec<CommentModel>>>,
}

impl TodoState {
    pub fn new() -> TodoState {
        let view = Value::new(View::Main);

        let posts = LazyCache::new(10 * 60 * 60 * 1000, || async move {
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
                LazyCache::new(10 * 60 * 60 * 1000, bind!(post_id, || async move {
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
                }))
            }
        });

        TodoState {
            view,
            posts,
            comments,
        }
    }
}
