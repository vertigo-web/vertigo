use std::{fmt::Display};

mod builder;
mod task;
mod process_result;

pub use builder::CommandBuilder;

#[derive(Clone)]
pub struct CommandLog {
    command: String,
}

impl CommandLog {
    pub fn new(command: String) -> Self {
        Self {
            command
        }
    }

    pub fn error(&self, error: impl Into<String>) -> CommandError {
        CommandError {
            command: self.command.clone(),
            error: error.into()
        }
    }

    pub fn map(&self, map: Option<fn(String) -> String>) -> Self {
        let command = match map {
            Some(log_format) => log_format(self.command.clone()),
            None => self.command.clone(),
        };

        Self {
            command
        }
    }

    pub fn to_string(self) -> String {
        self.command
    }
}

#[derive(Debug, Clone)]
pub struct CommandError {
    command: String,
    error: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { command, error } = self;
        f.write_fmt(
            format_args!("CommandError command=({command}), error={error}")
        )
    }
}

impl CommandError {
    fn new(command: impl Into<String>, error: impl Into<String>) -> CommandError {
        CommandError {
            command: command.into(),
            error: error.into(),
        }
    }
}

