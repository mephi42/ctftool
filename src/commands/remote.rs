use clap::Clap;

use anyhow::{anyhow, Result};

use crate::ctf;
use crate::git;

#[derive(Clap)]
pub struct Remote {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
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
}

#[derive(Clap)]
pub struct Show {}

#[derive(Clap)]
pub struct Add {
    /// Remote name
    pub name: String,

    /// Remote URL
    pub url: String,
}

#[derive(Clap)]
pub struct Rm {
    /// Remote name
    pub name: String,
}

pub fn run(remote: Remote) -> Result<()> {
    let mut ctf = ctf::load()?;
    match remote.subcmd {
        SubCommand::Show(_show) => {
            for remote in ctf.remotes {
                println!("{}", remote.name)
            }
        }
        SubCommand::Add(add) => {
            let existing = ctf.remotes.iter().find(|remote| remote.name == add.name);
            if existing.is_some() {
                return Err(anyhow!("Remote {} already exists", add.name));
            }
            let message = format!("Add remote {} pointing to {}", add.name, add.url);
            ctf.remotes.push(ctf::Remote {
                name: add.name,
                url: add.url,
            });
            git::commit(&ctf, &message)?;
        }
        SubCommand::Rm(rm) => {
            let message = format!("Remove remote {}", rm.name);
            let n_remotes = ctf.remotes.len();
            ctf.remotes.retain(|remote| remote.name != rm.name);
            if ctf.remotes.len() + 1 != n_remotes {
                return Err(anyhow!("Remote {} does not exist", rm.name));
            }
            git::commit(&ctf, &message)?;
        }
    }
    Ok(())
}
