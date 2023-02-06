#![allow(dead_code)]
use std::process::Stdio;
use std::{collections::HashMap};
use tokio::process::Command;
use tokio::process::{ChildStdout};
use std::fs::File;

use super::{CommandError, CommandLog};
use super::task;
use super::process_result;
use self::{task::{ProcessOwner}, process_result::ProcessResult};


enum Stdin {
    Null,
    Inherit,
    OtherProcess(ChildStdout),
}
enum Stdout {
    Display,
    Piped,
    File(File),
}

pub struct CommandBuilder {
    keep_process: Vec<ProcessOwner>,
    bin: String,
    params: Vec<String>,
    current_dir: Option<String>,
    env: HashMap<String, String>,
    stdin: Stdin,
    stderr: Stdout,
    stdout: Stdout,
    log_format: Option<fn(String) -> String>,
}

impl CommandBuilder {
    #[must_use]
    pub fn new(params: impl Into<String>) -> Self {
        let params_str: String = params.into();

        let mut params = params_str.split_whitespace();

        let bin = params.next().unwrap().to_string();
        let params = params.map(|item| item.to_string()).collect::<Vec<_>>();

        CommandBuilder {
            keep_process: Vec::new(),
            bin,
            params,
            current_dir: None,
            env: HashMap::new(),
            stdin: Stdin::Null,
            stderr: Stdout::Piped,
            stdout: Stdout::Piped,
            log_format: None,
        }
    }

    #[must_use]
    pub fn add_param(mut self, param: impl Into<String>) -> Self {
        let param = param.into();
        self.params.push(param);
        self
    }

    #[must_use]
    pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(name.into(), value.into());
        self
    }

    #[must_use]
    pub fn current_dir(mut self, current: impl Into<String>) -> Self {
        let current = current.into();
        self.current_dir = Some(current);
        self
    }

    pub async fn run(mut self) -> Result<(), CommandError> {
        self.stdin = Stdin::Null;
        self.stderr = Stdout::Display;
        self.stdout = Stdout::Display;

        let (process, _) = self.start_process()?;
        process.expect_success().await?;

        Ok(())
    }

    #[must_use]
    pub async fn output(mut self) -> Result<String, CommandError> {
        self.stdin = Stdin::Null;
        self.stderr = Stdout::Piped;
        self.stdout = Stdout::Piped;

        let (process, response) = self.start_process()?;
        process.expect_success().await?;

        let response = response.output().await?;
        Ok(response)
    }

    pub fn spawn(mut self) -> Result<(ProcessOwner, ProcessResult), CommandError> {
        self.stdin = Stdin::Null;
        self.stderr = Stdout::Piped;
        self.stdout = Stdout::Piped;
        self.log_format = Some(|command_str: String| {
            format!("spawn: {command_str}")
        });

        self.start_process()
    }

    pub fn run_child(mut self) -> Result<ProcessOwner, CommandError> {
        self.stdin = Stdin::Null;
        self.stderr = Stdout::Display;
        self.stdout = Stdout::Display;
        self.log_format = Some(|command_str: String| {
            format!("run child: {command_str}")
        });

        let (process, _) = self.start_process()?;
        Ok(process)
    }

    #[must_use]
    pub fn piped(mut self, params: impl Into<String>) -> Result<Self, CommandError> {
        self.stdin = Stdin::Null;
        self.stderr = Stdout::Display;
        self.stdout = Stdout::Piped;

        self.log_format = Some(|command_str: String| {
            format!("spawn: {command_str}")
        });

        let (process, response) = self.start_process()?;
        let (stdout, _) = response.split();
        let stdout = stdout.unwrap();

        let mut new_command = CommandBuilder::new(params);
        new_command.keep_process.push(process);
        new_command.stdin = Stdin::OtherProcess(stdout);
        Ok(new_command)
    }

    pub async fn output_to_file(mut self, path: impl Into<String>) -> Result<(), CommandError> {
        let path = path.into();
        let file = File::create(&path).map_err(|error| {
            let command = self.log();
            command.error(format!("Error opening file={path} error={error}"))
        })?;

        self.stdin = Stdin::Null;
        self.stderr = Stdout::Display;
        self.stdout = Stdout::File(file);

        let (process, _) = self.start_process()?;
        process.expect_success().await?;

        Ok(())
    }


    pub fn start_process(self) -> Result<(ProcessOwner, ProcessResult), CommandError> {
        let command_log = self.log();

        let mut command = Command::new(&self.bin);
        command.args(&self.params);

        if let Some(current) = &self.current_dir {
            command.current_dir(current);
        }

        for (name, value) in self.env.clone().into_iter() {
            command.env(name, value);
        }

        command.kill_on_drop(true);

        match self.stdin {
            Stdin::Null => {},
            Stdin::Inherit => {
                command.stdin(Stdio::inherit());
            },
            Stdin::OtherProcess(stdin) => {
                let a: Stdio = stdin.try_into().map_err(|error| {
                    command_log.error(format!("Stdin conversion error => {error}"))
                })?;

                command.stdin(a);
            }
        }
    
        let stdout_pipe = match self.stdout {
            Stdout::Display => {
                command.stdout(Stdio::inherit());
                false
            },
            Stdout::Piped => {
                command.stdout(Stdio::piped());
                true
            },
            Stdout::File(file) => {
                command.stdout(file);
                false
            }
        };

        let stderr_pipe = match self.stderr {
            Stdout::Display => {
                command.stderr(Stdio::inherit());
                false
            },
            Stdout::Piped => {
                command.stderr(Stdio::piped());
                true
            },
            Stdout::File(file) => {
                command.stderr(file);
                false
            }
        };

        let mut child = command.spawn().map_err({
            let command_str = command_log.clone();

            move |error| {
                command_str.error(format!("Process spawn error={error}"))
            }
        })?;

        let log = command_log.map(self.log_format).to_string();

        log::info!("\n$ {log}");

        let stderr = match stderr_pipe {
            true => {
                let Some(stderr) = child.stderr.take() else {
                    return Err(command_log.error(format!("Problem with getting stderr")));
                };

                Some(stderr)
            },
            false => None,
        };

        let stdout = match stdout_pipe {
            true => {
                let Some(stdout) = child.stdout.take() else {
                    return Err(command_log.error(format!("Problem with getting stdout")));
                };
                Some(stdout)
            },
            false => None,
        };

        Ok((
            ProcessOwner::new(command_log.clone(), self.keep_process, child),
            ProcessResult::new(
                command_log,
                stderr,
                stdout
            )
        ))
    }


    fn log(&self) -> CommandLog {
        let mut out = Vec::new();

        if let Some(current) = &self.current_dir {
            out.push(format!("CURRENT_DIR={current}"));
        }

        for (key, value) in self.env.iter() {
            out.push(format!("env:{key}={value}"));
        }

        out.push(self.bin.clone());

        out.extend(self.params.clone().into_iter());

        CommandLog::new(out.join(" "))
    }
}



