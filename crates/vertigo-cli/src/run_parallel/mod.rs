use clap::Args;

use crate::command::{CommandBuilder, CommandError};
use tokio::sync::mpsc::{unbounded_channel};

#[derive(Args, Debug)]
pub struct RunParallelOpts {
    pub command: Vec<String>,
}

pub async fn run_parallel(options: RunParallelOpts) -> Result<(), i32> {

    loop {
        let aa = run_parallel_iteration(&options).await;

        println!("end iteration status={aa:#?}");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

pub async fn run_parallel_iteration(options: &RunParallelOpts) -> Result<(), CommandError> {
    // let list = CommandBuilder::new("ls -al")
    //     .piped("gzip")?
    //     .output_to_file("ls.gzip").await?;

    // // for line in list.lines() {
    // //     println!("line = {line}");
    // // }

    // let d = CommandBuilder::new("cargo run --bin vertigo-demo-server")
        // .run().await?;
    let (pr, _) = CommandBuilder::new("cargo make demo-serve-api")
        .spawn()?;

    let aaa = pr.status().await?;

    let ggg = aaa.map(|status| {
        status.code()
    });

    println!("aaa = {aaa:#?}");
    println!("bbb = {ggg:#?}");
    todo!();

    // println!("");
    // println!("");

    // println!("list ===> {list:?}");


    println!("uruchamiam nową iterację 1: {options:?}");
    log::info!("uruchamiam nową iterację 2: {options:?}");

    let (sender, mut receiver) = unbounded_channel::<()>();

    let mut childs = Vec::new();

    for command in options.command.iter() {
        println!("command child parallel => {command}");

        let owner = CommandBuilder::new(command).run_child()?;

        let done = owner.when_done();
        tokio::spawn({
            let sender = sender.clone();

            async move {
                done.done().await;
                let _ = sender.send(());
            }
        });
    
        childs.push(owner);
    }

    println!("oczekuję na zakończenie któregokolwiek procesu");

    let _ = receiver.recv().await;

    for child in childs {
        child.off();
    }

    Ok(())
}

