use futures::stream::{Stream, StreamExt};
use poem::{
    handler,
    web::{
        sse::{Event, SSE},
        Data,
    },
};
use std::task::Poll;
use tokio::sync::watch::Receiver;

use crate::commons::spawn::SpawnOwner;

use super::Status;

#[handler]
pub fn handler_sse(state: Data<&Receiver<Status>>) -> SSE {
    let Data(state) = state;

    let stream = MyStream::new(state.clone()).map(|item| match item {
        Status::Building => Event::message("Building"),
        Status::Errors => Event::message("Errors"),
        Status::Version(version) => Event::message(format!("Version = {version}")),
    });

    SSE::new(stream)
}

struct MyStream<T: Default + Send + Sync + Unpin + Clone + PartialEq + 'static> {
    rx: Receiver<T>,
    last_emit_value: Option<T>,
    spawn: Option<SpawnOwner>,
}

impl<T: Default + Send + Sync + Unpin + Clone + PartialEq + 'static> MyStream<T> {
    pub fn new(rx: Receiver<T>) -> MyStream<T> {
        MyStream {
            rx,
            last_emit_value: None,
            spawn: None,
        }
    }
}

impl<T: Default + Send + Sync + Unpin + Clone + PartialEq + 'static> Stream for MyStream<T> {
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.spawn.is_none() {
            let waker = cx.waker().clone();
            let mut rx = self.rx.clone();

            self.spawn = Some(SpawnOwner::new(async move {
                loop {
                    rx.changed().await.unwrap();
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
        Poll::Ready(Some(value))
    }
}
