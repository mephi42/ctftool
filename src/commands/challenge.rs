use anyhow::{bail, Result};
use clap::Parser;

use crate::{ctf, git};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Challenge {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Prints a list of challenges and their descriptions
    #[clap(name = "show")]
    Show(Show),

    /// Adds a new challenge
    #[clap(name = "add")]
    Add(Add),

    /// Sets a challenge description
    #[clap(name = "set-description")]
    SetDescription(SetDescription),

    /// Removes an existing challenge
    #[clap(name = "rm")]
    Rm(Rm),
}

#[derive(Parser)]
pub struct Show {}

#[derive(Parser)]
pub struct Add {
    /// Challenge name
    pub name: String,
}

#[derive(Parser)]
pub struct SetDescription {
    /// Challenge name
    pub name: String,

    /// Description
    pub description: String,
}

#[derive(Parser)]
pub struct Rm {
    /// Challenge name
    pub name: String,
}

pub fn run(challenge: Challenge, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    match challenge.subcmd {
        SubCommand::Show(_show) => {
            for challenge in context.ctf.challenges {
                println!("{} - {}", challenge.name, challenge.description)
            }
        }
        SubCommand::Add(add) => {
            let existing = context
                .ctf
                .challenges
                .iter()
                .find(|challenge| challenge.name == add.name);
            if existing.is_some() {
                bail!("Challenge {} already exists", add.name);
            }
            let challenge_dir = context.root.join(&add.name);
            if !challenge_dir.exists() {
                bail!("Directory {} does not exist", challenge_dir.display());
            }
            let message = format!("Add challenge {}", add.name);
            context.ctf.challenges.push(ctf::Challenge {
                name: add.name,
                description: "".to_string(),
                binaries: Vec::new(),
                services: Vec::new(),
            });
            git::commit(&context, &message)?;
        }
        SubCommand::SetDescription(set_description) => {
            let message = format!(
                "Set challenge {} description to {}",
                set_description.name, set_description.description
            );
            let challenge = ctf::find_challenge_mut(&mut context.ctf, &set_description.name)?;
            challenge.description = set_description.description;
            git::commit(&context, &message)?;
        }
        SubCommand::Rm(rm) => {
            let message = format!("Remove challenge {}", rm.name);
            let n_challenges = context.ctf.challenges.len();
            context
                .ctf
                .challenges
                .retain(|challenge| challenge.name != rm.name);
            if context.ctf.challenges.len() + 1 != n_challenges {
                bail!("Challenge {} does not exist", rm.name);
            }
            git::commit(&context, &message)?;
        }
    }
    Ok(())
}
