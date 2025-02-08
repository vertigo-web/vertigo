use crate::commons::{command::CommandRun, ErrorCode};

fn is_target_instaled() -> Result<bool, ErrorCode> {
    let target_list = CommandRun::new("rustup target list")
        .set_error_code(ErrorCode::BuildPrerequisitesFailed)
        .output()?;

    let list = target_list.as_str().lines();
    for line in list {
        if line.contains("wasm32-unknown-unknown") {
            return Ok(line.contains("installed"));
        }
    }

    Ok(false)
}

pub fn check_env() -> Result<(), ErrorCode> {
    if is_target_instaled()? {
        return Ok(());
    }

    CommandRun::new("rustup target add wasm32-unknown-unknown")
        .set_error_code(ErrorCode::BuildPrerequisitesFailed)
        .run()
}
