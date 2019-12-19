use std::process::Command;

use anyhow::Result;

use crate::ctf;
use crate::ctf::CTF;
use crate::subprocess::check_call;

pub fn commit(ctf: &CTF, message: &str) -> Result<()> {
    ctf::store(ctf)?;
    check_call(Command::new("git").args(&["add", "."]))?;
    check_call(Command::new("git").args(&["commit", "-m", message]))?;
    Ok(())
}
