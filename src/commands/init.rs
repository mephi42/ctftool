use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::{anyhow, Result};

use crate::ctf;
use crate::git;
use crate::os_str::os_str_to_str;

#[derive(Parser)]
pub struct Init {}

pub fn run(_init: Init, root: PathBuf) -> Result<()> {
    let os_name = Path::file_name(&root)
        .ok_or_else(|| anyhow!("Could not obtain the name of the current directory"))?;
    let ctf = ctf::CTF {
        name: os_str_to_str(os_name)?.into(),
        remotes: vec![],
        challenges: vec![],
    };
    git::init(&root)?;
    if git::get_option(&root, "user.name")?.is_none() {
        git::set_option(&root, "user.name", "ctf")?;
    }
    if git::get_option(&root, "user.email")?.is_none() {
        git::set_option(&root, "user.email", "ctf@localhost")?;
    }
    let cwd = root.clone();
    let context = ctf::Context {
        ctf,
        credentials: ctf::Credentials::default(),
        root,
        path: Vec::new(),
        cwd,
    };
    git::commit(&context, "Initial commit")?;
    Ok(())
}
