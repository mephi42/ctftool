use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Serialize, Deserialize)]
pub struct CTF {
    pub name: String,
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
