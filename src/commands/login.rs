use std::io::{stdin, stdout, Write};

use clap::Parser;

use anyhow::{anyhow, Result};

use crate::ctf;
use crate::engines;
use crate::git;
use crate::http;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Login {
    /// Remote name
    #[clap(default_value = "origin")]
    pub name: String,
}

pub async fn run(login: Login, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    let remote = ctf::find_remote_mut(&mut context.ctf, &login.name)?;
    print!("Login: ");
    stdout().flush()?;
    let mut username = String::new();
    stdin().read_line(&mut username)?;
    username.truncate(username.trim_end().len());
    let password = rpassword::prompt_password("Password: ")?;
    let client = http::mk_client(&remote.rewrite_rules)?;
    if remote.engine == "auto" {
        remote.engine = engines::detect(&client, remote).await?;
    }
    let engine = engines::get_engine(&remote.engine)?;
    let cookie_store = engine.login(&client, remote, &username, &password).await?;
    let mut cookies = Vec::new();
    cookie_store
        .save_json(&mut cookies)
        .map_err(|_| anyhow!("Could not save cookies"))?;
    let message = &format!("Log into {}", login.name);
    ctf::set_cookies(
        &mut context.credentials,
        login.name,
        String::from_utf8(cookies)?,
    );
    git::commit(&context, message)?;
    Ok(())
}
