//! Local stand-ins for the two public APIs the demo talks to.
//!
//! The Todo tab reads `jsonplaceholder.typicode.com` and the Github Explorer reads
//! `api.github.com`. Both are fine when you are running the demo by hand, and both are the
//! wrong thing to put in a test: rate limits, DNS and someone else's uptime decide whether it
//! passes. The demo app takes each base URL from `env` (see `demo/app/src/app/api.rs`), so
//! pointing it here is a matter of two `--env` flags.
//!
//! The payload shapes mirror what the demo deserializes - `PostModel`, `CommentModel` and
//! `Branch` in `demo/app/src/app/{todo,github_explorer}/state.rs`. Only the fields the demo
//! actually reads are emitted.
//!
//! These routes are always on. They cost nothing when unused, and they let the whole demo run
//! with no internet.

use actix_web::{HttpResponse, Responder, http::header, web};
use serde::Serialize;
use serde_json::json;

/// How many posts `/todo/posts` returns, and how many comments each post has.
///
/// The browser test asserts against both, so they are named rather than spelled inline.
pub const POST_COUNT: u32 = 5;
pub const COMMENTS_PER_POST: u32 = 3;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/todo/posts", web::get().to(todo_posts))
        .route(
            "/todo/posts/{id}/comments",
            web::get().to(todo_post_comments),
        )
        .route(
            "/github/repos/{owner}/{repo}/branches/{branch}",
            web::get().to(github_branch),
        );
}

/// Cross-origin because the page is served by `vertigo serve` on one port and this API lives
/// on another. These are plain GETs with no custom headers, so a wildcard allow-origin is the
/// whole of what is needed - no preflight is involved.
fn cors_json(body: impl Serialize) -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .json(body)
}

async fn todo_posts() -> impl Responder {
    let posts = (1..=POST_COUNT)
        .map(|id| {
            json!({
                "id": id,
                "title": format!("stub post {id}"),
                "body": format!("body of stub post {id}"),
            })
        })
        .collect::<Vec<_>>();

    cors_json(posts)
}

async fn todo_post_comments(path: web::Path<u32>) -> impl Responder {
    let post_id = path.into_inner();

    let comments = (1..=COMMENTS_PER_POST)
        .map(|n| {
            json!({
                "id": post_id * 100 + n,
                "body": format!("comment {n} on post {post_id}"),
                "email": format!("commenter{n}@example.com"),
                "name": format!("stub commenter {n}"),
            })
        })
        .collect::<Vec<_>>();

    cors_json(comments)
}

/// The sha the Github Explorer renders once a repo is fetched. Fixed, so the test can assert
/// on it exactly.
pub const BRANCH_SHA: &str = "0000000000000000000000000000000000000001";

async fn github_branch(path: web::Path<(String, String, String)>) -> impl Responder {
    let (owner, repo, branch) = path.into_inner();

    let signature = json!({
        "name": "Stub Author",
        "email": "stub@example.com",
    });

    cors_json(json!({
        "name": branch,
        "commit": {
            "sha": BRANCH_SHA,
            "commit": {
                "author": signature,
                "committer": signature,
            },
        },
        // Not read by the demo; here so a glance at the response says where it came from.
        "_stub_repo": format!("{owner}/{repo}"),
    }))
}
