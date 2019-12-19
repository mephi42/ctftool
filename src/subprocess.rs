use std::process::Command;

use anyhow::{anyhow, Result};

pub fn check_call(command: &mut Command) -> Result<()> {
    let status = command.spawn()?.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Process exited with non-zero code: {}", status))
    }
}
