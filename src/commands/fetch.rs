use clap::Clap;

use anyhow::Result;

use crate::ctf;
use crate::engines;
use crate::git;

#[derive(Clap)]
pub struct Fetch {
    /// Remote name
    #[clap(default_value = "origin")]
    pub name: String,
}

pub async fn run(fetch: Fetch) -> Result<()> {
    let mut ctf = ctf::load()?.ctf;
    let mut remote = ctf::find_remote_mut(&mut ctf, &fetch.name)?;
    let fetched = if remote.engine == "auto" {
        let (engine, fetched) = engines::fetch_auto(&remote).await?;
        remote.engine = engine;
        fetched
    } else {
        let engine = engines::get_engine(&remote.engine)?;
        engine.fetch(&remote).await?
    };
    ctf::merge(&mut ctf, fetched);
    git::commit(&ctf, &format!("Fetch from {}", fetch.name))?;
    Ok(())
}
