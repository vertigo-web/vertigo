use clap::Args;

use crate::command::CommandBuilder;
use tokio::sync::mpsc::unbounded_channel;
use std::thread::spawn;

#[derive(Args, Debug)]
pub struct RunParallelOpts {
    pub command: Vec<String>,
}

pub async fn run_parallel(options: RunParallelOpts) -> Result<(), i32> {

    let mut childs = Vec::new();

    let (sender, receiver) = unbounded_channel::<()>();

    for command in options.command {
        match CommandBuilder::new(command).run_child() {
            Ok(child) => {
                childs.push(child);
            },
            Err(err) => {
                sender.send(()).unwrap();
            }
        }
    }

    // spawn(|| {

    // });

    for child in childs {
        // let child = child.clone();

        spawn({
            let receiver = sender.subscribe();
            move || {
            }
        });
    }

    println!("options {options:#?}");

    todo!()
}