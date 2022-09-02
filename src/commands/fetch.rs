use clap::Parser;
use cookie_store::CookieStore;

use anyhow::{anyhow, Result};

use crate::ctf;
use crate::engines;
use crate::git;
use crate::http;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Fetch {
    /// Remote name
    #[clap(default_value = "origin")]
    pub name: String,
}

pub async fn run(fetch: Fetch, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    let mut remote = ctf::find_remote_mut(&mut context.ctf, &fetch.name)?;
    let client = http::mk_client(&remote.rewrite_rules)?;
    let mut cookie_store = CookieStore::default();
    for remote_credentials in &context.credentials.remotes {
        if remote_credentials.name == fetch.name {
            cookie_store = CookieStore::load_json(remote_credentials.cookies.as_bytes())
                .map_err(|_| anyhow!("Could not load cookies"))?;
            break;
        }
    }
    if remote.engine == "auto" {
        remote.engine = engines::detect(&client, remote).await?;
    }
    let engine = engines::get_engine(&remote.engine)?;
    let fetched = engine.fetch(&client, &cookie_store, remote).await?;
    ctf::merge(&mut context.ctf, fetched);
    git::commit(&context, &format!("Fetch from {}", fetch.name))?;
    Ok(())
}
