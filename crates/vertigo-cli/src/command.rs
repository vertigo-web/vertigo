#![allow(dead_code)]
use std::{process::{exit, Output, Command, Stdio, ChildStdout}, collections::{HashMap}};

pub struct CommandRun {
    bin: String,
    params: Vec<String>,
    error_allowed: bool,
    current_dir: Option<String>,
    env: HashMap<String, String>,
    stdin: Option<ChildStdout>,
}

impl CommandRun {
    #[must_use]
    pub fn new(params: impl Into<String>) -> Self {
        let params_str: String = params.into();

        let mut params = params_str.split_whitespace();

        let bin = params.next().unwrap().to_string();
        let params = params.map(|item| item.to_string()).collect::<Vec<_>>();

        CommandRun {
            bin,
            params,
            error_allowed: false,
            current_dir: None,
            env: HashMap::new(),
            stdin: None,
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

    pub fn error_allowed(mut self, flag: bool) -> Self {
        self.error_allowed = flag;
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

        out.extend(self.params.clone().into_iter());

        out.join(" ")
    }

    // pub fn stdin(mut self, stdin: Stdio) -> Self {
    //     self.stdin = Some(stdin);
    //     self
    // }

    fn convert_to_string(data: Vec<u8>) -> String {
        String::from_utf8(data).unwrap()
    }

    fn create_command(self) -> (String, Command, Option<ChildStdout>) {
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

        (command_str, command, self.stdin)
    }

    fn show_status(error_allowed: bool, output: &Output) {
        if output.status.success() {
            //ok
        } else {
            let status = output.status;
    
            let status = format!("$ status = {status:?}");
            log::error!("$ status = {status:?}\n");

            if error_allowed {
                //ok
            } else {
                exit(1);
            }
        }
    }

    fn set_stdin(command: &mut Command, out: Option<ChildStdout>, get_default_stdio: Option<fn() -> Stdio>) {
        if let Some(out) = out {
            command.stdin(out);
            return;
        }

        if let Some(out_default) = get_default_stdio {
            command.stdin(out_default());
            return;
        }
    }

    pub fn run(self) {
        let error_allowed = self.error_allowed;
        let (command_str, mut command, stdin) = self.create_command();
        println!("{command_str}");

        Self::set_stdin(&mut command, stdin, Some(|| Stdio::inherit()));
        command.stderr(Stdio::inherit());
        command.stdout(Stdio::inherit());

        let out = command.output().unwrap();
        Self::show_status(error_allowed, &out);
    }

    #[must_use]
    pub fn output(self) -> String {
        let error_allowed = self.error_allowed;
        let (command_str, mut command, stdin) = self.create_command();
        println!("{command_str}");

        Self::set_stdin(&mut command, stdin, None);
        command.stderr(Stdio::inherit());

        let out = command.output().unwrap();
        Self::show_status(error_allowed, &out);

        Self::convert_to_string(out.stdout)
    }

    pub fn spawn(self) {
        let (command_str, mut command, stdin) = self.create_command();
        println!("spawn: {command_str}");

        Self::set_stdin(&mut command, stdin, Some(|| Stdio::null()));
        command.stderr(Stdio::null());
        command.stdout(Stdio::null());

        command.spawn().unwrap();
    }

    #[must_use]
    pub fn piped(self, params: impl Into<String>) -> Self {
        let (command_str, mut command, stdin) = self.create_command();
        println!("spawn: {command_str}");

        Self::set_stdin(&mut command, stdin, Some(|| Stdio::null()));
        command.stderr(Stdio::null());
        command.stdout(Stdio::piped());

        let child = command.spawn().unwrap();

        let mut new_command = CommandRun::new(params);
        let stdout = child.stdout.unwrap();
        new_command.stdin = Some(stdout);
        new_command
    }

    pub fn output_to_file(self, path: impl Into<String>) {
        let error_allowed = self.error_allowed;
        let (command_str, mut command, stdin) = self.create_command();
        println!("{command_str}");

        Self::set_stdin(&mut command, stdin, Some(|| Stdio::null()));
        command.stderr(Stdio::inherit());

        use std::fs::File;
        let path = path.into();
        let file = File::create(path).unwrap();
        command.stdout(file);

        let out = command.output().unwrap();
        Self::show_status(error_allowed, &out);
    }

}