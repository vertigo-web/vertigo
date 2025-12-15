use std::{
    collections::HashMap,
    fs::File,
    process::{Child, ChildStdout, Command, ExitStatus, Output, Stdio},
};

use crate::commons::ErrorCode;

pub struct CommandRun {
    bin: String,
    params: Vec<String>,
    error_code: Option<ErrorCode>,
    current_dir: Option<String>,
    env: HashMap<String, String>,
    child: Option<Child>,
}

impl CommandRun {
    #[must_use]
    pub fn new(params: impl Into<String>) -> Self {
        let params_str: String = params.into();

        let mut params = params_str.split_whitespace();

        let bin = params.next().unwrap_or_default().to_string();
        let params = params.map(|item| item.to_string()).collect::<Vec<_>>();

        CommandRun {
            bin,
            params,
            error_code: None,
            current_dir: None,
            env: HashMap::new(),
            child: None,
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
    #[allow(dead_code)]
    pub fn current_dir(mut self, current: impl Into<String>) -> Self {
        let current = current.into();
        self.current_dir = Some(current);
        self
    }

    pub fn set_error_code(mut self, error_code: ErrorCode) -> Self {
        self.error_code = Some(error_code);
        self
    }

    pub fn allow_error(mut self) -> Self {
        self.error_code = None;
        self
    }

    fn log(&self) -> String {
        let mut out = Vec::new();

        if let Some(current) = &self.current_dir {
            out.push(format!("CURRENT_DIR={current}"));
        }

        for (key, value) in self.env.iter() {
            out.push(format!("env:{key}={value}"));
        }

        out.push(self.bin.clone());

        out.extend(self.params.clone());

        out.join(" ")
    }

    fn convert_to_string(data: Vec<u8>) -> String {
        match String::from_utf8(data) {
            Ok(data) => data,
            Err(err) => {
                log::error!("Error encoding utf-8: {err}");
                "".to_string()
            }
        }
    }

    fn create_command(self) -> (String, Command, Option<Child>) {
        let command_str = format!("$ {}", self.log());
        // let command_color = log_message::green(command_str);
        let command_str = format!("\n{command_str}");

        let mut command = Command::new(&self.bin);
        command.args(&self.params);

        if let Some(current) = &self.current_dir {
            command.current_dir(current);
        }

        for (name, value) in self.env.clone().into_iter() {
            command.env(name, value);
        }

        (command_str, command, self.child)
    }

    fn intercept_status(error_code: Option<ErrorCode>, output: &Output) -> Result<(), ErrorCode> {
        if output.status.success() {
            Ok(())
        } else {
            log::error!("Subcommand finished with {}", output.status);

            if let Some(error_code) = error_code {
                Err(error_code)
            } else {
                Ok(())
            }
        }
    }

    fn set_stdin(
        command: &mut Command,
        out: Option<ChildStdout>,
        get_default_stdio: Option<fn() -> Stdio>,
    ) {
        if let Some(out) = out {
            command.stdin(out);
        } else if let Some(out_default) = get_default_stdio {
            command.stdin(out_default());
        }
    }

    pub fn run(self) -> Result<(), ErrorCode> {
        let error_code = self.error_code;
        let (command_str, mut command, mut child) = self.create_command();
        println!("{command_str}");

        if let Some(child) = child.as_mut() {
            Self::set_stdin(&mut command, child.stdout.take(), Some(Stdio::inherit));
        }
        command.stderr(Stdio::inherit());
        command.stdout(Stdio::inherit());

        let out = command.output().map_err(|err| {
            log::error!("Can't read child output: {err}");
            ErrorCode::CantSpawnChildProcess
        })?;

        if let Some(child) = child.as_mut() {
            if child.wait().is_err() {
                return Err(ErrorCode::CouldntWaitForChildProcess);
            }
        }

        Self::intercept_status(error_code, &out)?;

        Ok(())
    }

    pub fn output_with_status(self) -> Result<(ExitStatus, String), ErrorCode> {
        let error_code = self.error_code;
        let (command_str, mut command, mut child) = self.create_command();
        println!("{command_str}");

        if let Some(child) = child.as_mut() {
            Self::set_stdin(&mut command, child.stdout.take(), None);
        };
        command.stderr(Stdio::inherit());

        let out = command.output().map_err(|err| {
            log::error!("Can't read child output: {err}");
            ErrorCode::CantSpawnChildProcess
        })?;
        Self::intercept_status(error_code, &out)?;

        if let Some(child) = child.as_mut() {
            if child.wait().is_err() {
                return Err(ErrorCode::CouldntWaitForChildProcess);
            }
        }

        Ok((out.status, Self::convert_to_string(out.stdout)))
    }

    pub fn output(self) -> Result<String, ErrorCode> {
        self.output_with_status().map(|(_, output)| output)
    }

    #[allow(dead_code)]
    pub fn spawn(self) -> Result<(Child, Option<Child>), ErrorCode> {
        let (command_str, mut command, mut child) = self.create_command();
        println!("spawn: {command_str}");

        if let Some(child) = child.as_mut() {
            Self::set_stdin(&mut command, child.stdout.take(), Some(Stdio::null));
        }
        command.stderr(Stdio::null());
        command.stdout(Stdio::null());

        Ok((
            command.spawn().map_err(|err| {
                log::error!("Can't spawn child process: {err}");
                ErrorCode::CantSpawnChildProcess
            })?,
            child,
        ))
    }

    #[allow(dead_code)]
    pub fn piped(self, params: impl Into<String>) -> Result<Self, ErrorCode> {
        let (command_str, mut command, mut child) = self.create_command();
        println!("spawn: {command_str}");

        if let Some(child) = child.as_mut() {
            Self::set_stdin(&mut command, child.stdout.take(), Some(Stdio::null));
        }
        command.stderr(Stdio::null());
        command.stdout(Stdio::piped());

        let child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                log::error!("Can't spawn child process: {err}");
                return Err(ErrorCode::CantSpawnChildProcess);
            }
        };

        let mut new_command = CommandRun::new(params);
        new_command.child = Some(child);
        Ok(new_command)
    }

    #[allow(dead_code)]
    pub fn output_to_file(self, path: impl Into<String>) -> Result<(), ErrorCode> {
        let error_code = self.error_code;
        let (command_str, mut command, child) = self.create_command();
        println!("{command_str}");

        if let Some(child) = child {
            Self::set_stdin(&mut command, child.stdout, Some(Stdio::null));
        }
        command.stderr(Stdio::inherit());

        let path = path.into();

        let file = match File::create(path.clone()) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Can't write to file {path}: {err}");
                return Err(ErrorCode::CantWriteOrRemoveFile);
            }
        };

        command.stdout(file);

        let out = command.output().map_err(|err| {
            log::error!("Can't read output: {err}");
            ErrorCode::CantSpawnChildProcess
        })?;
        Self::intercept_status(error_code, &out)?;

        Ok(())
    }
}
