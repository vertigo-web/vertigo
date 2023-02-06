use tokio::process::{ChildStdout, ChildStderr};

use super::{CommandError, CommandLog};

pub struct ProcessResult {
    command: CommandLog,
    stderr: Option<ChildStderr>,
    stdout: Option<ChildStdout>,
}

impl ProcessResult {
    pub fn new(
        command: CommandLog,
        stderr: Option<ChildStderr>,
        stdout: Option<ChildStdout>,
    ) -> Self {
        ProcessResult {
            command,
            stderr,
            stdout
        }
    }

    pub fn split(self) -> (Option<ChildStdout>, Option<ChildStderr>) {
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

    pub async fn output_with_err(self) -> Result<(String, String), CommandError> {
        use tokio::io::AsyncReadExt;

        let stdout = match self.stdout {
            Some(mut stdout) => {
                let mut result = String::new();

                match stdout.read_to_string(&mut result).await {
                    Ok(_) => {},
                    Err(error) => {
                        return Err(self.command.error(format!("Problem with reading stdout: {error:?}")));
                    }
                };

                result
            },
            None => {
                String::new()
            }
        };

        let stderr = match self.stderr {
            Some(mut stderr) => {
                let mut result = String::new();
                match stderr.read_to_string(&mut result).await  {
                    Ok(_) => {},
                    Err(error) => {
                        return Err(self.command.error(format!("Problem with reading stderr: {error:?}")));
                    }
                };
                result
            },
            None => {
                String::new()
            }
        };

        Ok((stdout, stderr))
    }
}
