use std::process::ExitStatus;
use tokio::process::{Child};
use super::{CommandError, CommandLog};
use futures::future::{
    abortable,
    AbortHandle,
};
use tokio::sync::watch;

pub struct CommandProcessAbort {
    abort: AbortHandle,
}

impl CommandProcessAbort {
    pub fn abort(self) {
        self.abort.abort();
    }
}

pub struct ProcessOwner {
    _keep_process: Vec<ProcessOwner>, 
    status: watch::Receiver<Option<Result<Option<ExitStatus>, CommandError>>>,
    abort: AbortHandle,
}

impl ProcessOwner {
    pub fn new(log: CommandLog, keep_process: Vec<ProcessOwner>, mut child: Child) -> Self {
        let (sender, status) = watch::channel(None);

        let (fut_ab, abort) = abortable(async move {
            child.wait().await
        });

        tokio::spawn(async move {
            let result = fut_ab.await;
    
            match result {
                Ok(Ok(status)) => {
                    let _ = sender.send(Some(Ok(Some(status))));
                }
                Ok(Err(err)) => {
                    let _ = sender.send(Some(Err(log.error(format!("{err}")))));
                },
                Err(err) => {
                    let _ = sender.send(Some(Ok(None)));
                }
            }
        });

        Self {
            _keep_process: keep_process,
            status,
            abort,
        }
    }

    #[must_use]
    fn get_abort(&self) -> CommandProcessAbort {
        CommandProcessAbort {
            abort: self.abort.clone()
        }
    }

    #[must_use]
    async fn status(self) -> Result<Option<ExitStatus>, CommandError> {
        let mut status = self.status.clone();

        loop {
            {
                let value = status.borrow();
                if let Some(value) = value.as_ref() {
                    return (*value).clone();
                }
            }
            
            let aaa = status.changed().await;
        }
    }

    #[must_use]
    pub async fn expect_success(self) -> Result<(), CommandError> {
        todo!()
    }

}

impl Drop for ProcessOwner {
    fn drop(&mut self) {
        self.abort.abort();
    }
}
