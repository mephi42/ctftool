use std::process::{Command, Stdio};

use anyhow::Result;

use crate::ctf;
use crate::subprocess::check_call;

pub fn commit(context: &ctf::Context, message: &str) -> Result<()> {
    ctf::store(context)?;
    check_call(Command::new("git").args(&["add", "."]))?;
    check_call(Command::new("git").args(&["commit", "-m", message]))?;
    Ok(())
}

pub fn get_option(name: &str) -> Result<Option<String>> {
    let child = Command::new("git")
        .args(&["config", name])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(Some(String::from_utf8(output.stdout)?))
    } else {
        Ok(None)
    }
}

pub fn set_option(name: &str, value: &str) -> Result<()> {
    check_call(Command::new("git").args(&["config", name, value]))?;
    Ok(())
}
