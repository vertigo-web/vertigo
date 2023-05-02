use crate::command::CommandRun;

fn is_target_instaled() -> bool {
    let target_list = CommandRun::new("rustup target list").output();

    let list = target_list.as_str().lines();
    for line in list {
        if line.contains("wasm32-unknown-unknown") {
            return line.contains("installed");
        }
    }

    false
}

pub fn check_env() -> Result<(), i32> {
    if is_target_instaled() {
        return Ok(());
    }

    CommandRun::new("rustup target add wasm32-unknown-unknown").run();
    Ok(())
}
