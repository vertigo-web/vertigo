use crate::command::{CommandBuilder, CommandError};

async fn is_target_instaled() -> Result<bool, CommandError> {
    
    let target_list = CommandBuilder::new("rustup target list").output().await?;

    let list = target_list.as_str().lines();
    for line in list {
        if line.contains("wasm32-unknown-unknown") {
            return Ok(line.contains("installed"));
        }
    }

    Ok(false)
}

pub async fn check_env_async() -> Result<(), CommandError> {
    /*
    if [ "$(rustup target list | grep wasm32-unknown-unknown | grep installed)" = "" ];
    then
        rustup target add wasm32-unknown-unknown;
    fi
    */

    if is_target_instaled().await? {
        return Ok(());
    }

    CommandBuilder::new("rustup target add wasm32-unknown-unknown").run().await?;
    Ok(())
}

pub async fn check_env() -> Result<(), i32> {
    match check_env_async().await {
        Ok(()) => Ok(()),
        Err(error) => {
            log::error!("error in running diagnostic functions {error}");
            Ok(())
        }
    }
}
