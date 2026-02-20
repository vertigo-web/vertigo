use actix_web::dev::ServerHandle;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    signal,
    task::{self, JoinHandle},
};

pub struct SpawnOwner {
    handler: JoinHandle<()>,
}

impl SpawnOwner {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> SpawnOwner {
        SpawnOwner {
            handler: task::spawn(future),
        }
    }

    pub fn off(self) {}
}

impl Drop for SpawnOwner {
    fn drop(&mut self) {
        self.handler.abort();
    }
}

#[derive(Clone)]
pub(crate) struct ServerOwner {
    pub handle: ServerHandle,
}

impl std::future::Future for ServerOwner {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for ServerOwner {
    fn drop(&mut self) {
        log::info!("Stopping server...");
        #[allow(clippy::let_underscore_future)]
        let _ = self.handle.stop(false);
    }
}

pub async fn term_signal() -> &'static str {
    let Ok(mut sigterm) = signal::unix::signal(signal::unix::SignalKind::terminate()) else {
        log::error!("Can't register signal handler");
        return "Error";
    };

    tokio::select! {
        _ = signal::ctrl_c() => "Ctrl+C",
        _ = sigterm.recv() => "SIGTERM",
    }
}
