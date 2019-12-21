use std::env::current_dir;
use std::path::Path;
use std::process::Command;

use clap::Clap;

use anyhow::{anyhow, Result};

use crate::ctf::CTF;
use crate::git;
use crate::subprocess::check_call;

#[derive(Clap)]
pub struct Init {}

pub fn run(_init: Init) -> Result<()> {
    let ctf = CTF {
        name: Path::file_name(&current_dir()?)
            .and_then({ |x| x.to_str() })
            .ok_or_else({ || anyhow!("Could not obtain the name of the current directory") })?
            .into(),
        remotes: vec![],
        challenges: vec![],
    };
    check_call(Command::new("git").args(&["init"]))?;
    if git::get_option("user.name")?.is_none() {
        git::set_option("user.name", "ctf")?;
    }
    if git::get_option("user.email")?.is_none() {
        git::set_option("user.email", "ctf@localhost")?;
    }
    git::commit(&ctf, "Initial commit")?;
    Ok(())
}
