use anyhow::{bail, Result};
use clap::Parser;

use crate::path::path_to_str;
use crate::path::relativize;
use crate::{ctf, git};
use std::path::{Path, PathBuf};

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

/// Resolve challenge name
fn resolve(root: &Path, cwd: &Path, s: String) -> Result<String> {
    let (_, relative_path) = relativize(root, cwd, PathBuf::from(s))?;
    path_to_str(&relative_path).map(|s| s.into())
}

pub fn run(challenge: Challenge, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir.clone())?;
    match challenge.subcmd {
        SubCommand::Show(_show) => {
            for challenge in context.ctf.challenges {
                println!("{} - {}", challenge.name, challenge.description)
            }
        }
        SubCommand::Add(add) => {
            let name = resolve(&context.root, &current_dir, add.name)?;
            let existing = context
                .ctf
                .challenges
                .iter()
                .find(|challenge| challenge.name == name);
            if existing.is_some() {
                bail!("Challenge {} already exists", &name);
            }
            let challenge_dir = context.root.join(&name);
            if !challenge_dir.exists() {
                bail!("Directory {} does not exist", challenge_dir.display());
            }
            let message = format!("Add challenge {}", name);
            context.ctf.challenges.push(ctf::Challenge {
                name,
                description: "".to_string(),
                binaries: Vec::new(),
                services: Vec::new(),
            });
            git::commit(&context, &message)?;
        }
        SubCommand::SetDescription(set_description) => {
            let name = resolve(&context.root, &current_dir, set_description.name)?;
            let message = format!(
                "Set challenge {} description to {}",
                name, set_description.description
            );
            let challenge = ctf::find_challenge_mut(&mut context.ctf, &name)?;
            challenge.description = set_description.description;
            git::commit(&context, &message)?;
        }
        SubCommand::Rm(rm) => {
            let name = resolve(&context.root, &current_dir, rm.name)?;
            let message = format!("Remove challenge {}", name);
            let n_challenges = context.ctf.challenges.len();
            context
                .ctf
                .challenges
                .retain(|challenge| challenge.name != name);
            if context.ctf.challenges.len() + 1 != n_challenges {
                bail!("Challenge {} does not exist", &name);
            }
            git::commit(&context, &message)?;
        }
    }
    Ok(())
}
