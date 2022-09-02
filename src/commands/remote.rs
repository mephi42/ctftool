use anyhow::{bail, Result};
use clap::Parser;

use crate::ctf;
use crate::git;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Remote {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Prints a list of configured link names
    #[clap(name = "show")]
    Show(Show),

    /// Adds a new link
    #[clap(name = "add")]
    Add(Add),

    /// Removes an existing link
    #[clap(name = "rm")]
    Rm(Rm),

    /// Shows an associated engine
    #[clap(name = "get-engine")]
    GetEngine(GetEngine),

    /// Sets an associated engine
    #[clap(name = "set-engine")]
    SetEngine(SetEngine),
}

#[derive(Parser)]
pub struct Show {}

#[derive(Parser)]
pub struct Add {
    /// Remote name
    pub name: String,

    /// Remote URL
    pub url: String,
}

#[derive(Parser)]
pub struct Rm {
    /// Remote name
    pub name: String,
}

#[derive(Parser)]
pub struct GetEngine {
    /// Remote name
    pub name: String,
}

#[derive(Parser)]
pub struct SetEngine {
    /// Remote name
    pub name: String,

    /// Engine name
    pub engine: String,
}

pub fn run(remote: Remote, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    match remote.subcmd {
        SubCommand::Show(_show) => {
            for remote in context.ctf.remotes {
                println!("{}", remote.name)
            }
        }
        SubCommand::Add(add) => {
            let existing = context
                .ctf
                .remotes
                .iter()
                .find(|remote| remote.name == add.name);
            if existing.is_some() {
                bail!("Remote {} already exists", add.name);
            }
            let message = format!("Add remote {} pointing to {}", add.name, add.url);
            context.ctf.remotes.push(ctf::Remote {
                name: add.name,
                url: add.url,
                engine: ctf::default_engine(),
                rewrite_rules: Vec::new(),
            });
            git::commit(&context, &message)?;
        }
        SubCommand::Rm(rm) => {
            let message = format!("Remove remote {}", rm.name);
            let n_remotes = context.ctf.remotes.len();
            context.ctf.remotes.retain(|remote| remote.name != rm.name);
            if context.ctf.remotes.len() + 1 != n_remotes {
                bail!("Remote {} does not exist", rm.name);
            }
            git::commit(&context, &message)?;
        }
        SubCommand::GetEngine(get_engine) => {
            let remote = ctf::find_remote(&context.ctf, &get_engine.name)?;
            println!("{}", remote.engine);
        }
        SubCommand::SetEngine(set_engine) => {
            let message = format!(
                "Set remote {} engine to {}",
                set_engine.name, set_engine.engine
            );
            let remote = ctf::find_remote_mut(&mut context.ctf, &set_engine.name)?;
            remote.engine = set_engine.engine;
            git::commit(&context, &message)?;
        }
    }
    Ok(())
}
