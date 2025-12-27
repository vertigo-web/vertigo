use actix_web::{web, HttpResponse, Responder};
use futures::stream::Stream;
use std::task::Poll;
use tokio::sync::watch::Receiver;

use crate::commons::spawn::SpawnOwner;

use super::Status;

pub async fn handler_sse(state: web::Data<Receiver<Status>>) -> impl Responder {
    let stream = MyStream::new(state.get_ref().clone());

    HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .streaming(stream)
}

struct MyStream {
    rx: Receiver<Status>,
    last_emit_value: Option<Status>,
    spawn: Option<SpawnOwner>,
}

impl MyStream {
    pub fn new(rx: Receiver<Status>) -> MyStream {
        MyStream {
            rx,
            last_emit_value: None,
            spawn: None,
        }
    }
}

impl Stream for MyStream {
    type Item = Result<actix_web::web::Bytes, actix_web::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.spawn.is_none() {
            let waker = cx.waker().clone();
            let mut rx = self.rx.clone();

            self.spawn = Some(SpawnOwner::new(async move {
                loop {
                    if let Err(err) = rx.changed().await {
                        log::error!("{err}");
                    };
                    waker.wake_by_ref();
                }
            }));
        }

        let value = self.rx.borrow().clone();

        if let Some(last_emit_value) = &self.last_emit_value {
            if last_emit_value == &value {
                return Poll::Pending;
            }
        }

        self.last_emit_value = Some(value.clone());

        let message = match value {
            Status::Building => "data: Building\n\n".to_string(),
            Status::Errors => "data: Errors\n\n".to_string(),
            Status::Version(version) => format!("data: Version = {version}\n\n"),
        };

        Poll::Ready(Some(Ok(actix_web::web::Bytes::from(message))))
    }
}
