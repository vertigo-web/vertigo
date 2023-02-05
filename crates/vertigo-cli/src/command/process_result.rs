use tokio::process::{ChildStdout, ChildStderr};

use super::{CommandError, CommandLog};

pub struct ProcessResult {
    command: CommandLog,
    stderr: ChildStderr,
    stdout: ChildStdout,
}

impl ProcessResult {
    pub fn new(
        command: CommandLog,
        stderr: ChildStderr,
        stdout: ChildStdout,
    ) -> Self {
        ProcessResult {
            command,
            stderr,
            stdout
        }
    }

    pub fn split(self) -> (ChildStdout, ChildStderr) {
        (self.stdout, self.stderr)
    }

    pub async fn output(self) -> Result<String, CommandError> {
        let command = self.command.clone();
        let result = self.output_with_err().await;

        match result {
            Ok((stdout, stderr)) => {
                match stderr.is_empty() {
                    true => Ok(stdout),
                    false => Err(command.error("Empty stderr was expected")),
                }
            },
            Err(error) => Err(error)
        }
    }

    pub async fn output_with_err(mut self) -> Result<(String, String), CommandError> {
        use tokio::io::AsyncReadExt;

        let mut stdout = String::new();
        match self.stdout.read_to_string(&mut stdout).await {
            Ok(_) => {},
            Err(error) => {
                return Err(self.command.error(format!("Problem with reading stdout: {error:?}")));
            }
        };

        let mut stderr = String::new();
        match self.stderr.read_to_string(&mut stderr).await  {
            Ok(_) => {},
            Err(error) => {
                return Err(self.command.error(format!("Problem with reading stderr: {error:?}")));
            }
        };

        Ok((stdout, stderr))
    }
}
