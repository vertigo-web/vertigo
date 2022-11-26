use tokio::task::{self, JoinHandle};
use std::future::Future;

pub struct SpawnOwner {
    handler: JoinHandle<()>,
}

impl SpawnOwner {
    pub fn new(future: impl Future<Output=()> + Send + 'static) -> SpawnOwner {
        SpawnOwner {
            handler: task::spawn(future)
        }
    }

    pub fn off(self) {
    }
}

impl Drop for SpawnOwner {
    fn drop(&mut self) {
        self.handler.abort();
    }
}
