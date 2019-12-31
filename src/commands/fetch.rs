use clap::Clap;

use anyhow::{anyhow, Result};

use crate::ctf;
use crate::engines::watevr;
use crate::git;

#[derive(Clap)]
pub struct Fetch {
    /// Remote name
    #[clap(default_value = "origin")]
    pub name: String,
}

pub async fn run(fetch: Fetch) -> Result<()> {
    let mut ctf = ctf::load()?.ctf;
    let remote = ctf
        .remotes
        .iter()
        .find(|remote| remote.name == fetch.name)
        .ok_or_else(|| anyhow!("Remote {} does not exist", fetch.name))?;
    let fetched = watevr::fetch(&remote).await?;
    ctf::merge(&mut ctf, fetched);
    git::commit(&ctf, &format!("Fetch from {}", fetch.name))?;
    Ok(())
}
