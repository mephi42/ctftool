use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Serialize, Deserialize)]
pub struct CTF {
    pub name: String,
    pub remotes: Vec<Remote>,
}

#[derive(Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

pub fn load() -> Result<CTF> {
    let bytes = fs::read(".ctf")?;
    let str = &String::from_utf8(bytes)?;
    let ctf = serde_yaml::from_str(str)?;
    Ok(ctf)
}

pub fn store(ctf: &CTF) -> Result<()> {
    fs::write(
        ".gitignore",
        "/*
!/.ctf
!/.gitignore
",
    )?;
    fs::write(".ctf", serde_yaml::to_string(&ctf)?)?;
    Ok(())
}
