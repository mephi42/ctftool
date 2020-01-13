use clap::Clap;
use console::style;

use anyhow::{anyhow, bail, Result};

use crate::ctf;
use crate::git;
use crate::option;

#[derive(Clap)]
pub struct Binary {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Prints a list of registered binaries
    #[clap(name = "show")]
    Show(Show),

    /// Registers a new binary
    #[clap(name = "add")]
    Add(Add),

    /// Removes a registered binary
    #[clap(name = "rm")]
    Rm(Rm),

    /// Makes a registered binary a default one
    #[clap(name = "default")]
    Default(Default),
}

#[derive(Clap)]
pub struct Show {}

#[derive(Clap)]
pub struct Add {
    /// Name in `binary.alternative` format
    pub name: String,
}

#[derive(Clap)]
pub struct Rm {
    /// Name
    pub name: String,
}

#[derive(Clap)]
pub struct Default {
    /// Name
    pub name: String,
}

fn split(name: &str) -> Result<(&str, &str)> {
    match name.rfind('.') {
        Some(pos) => Ok((&name[..pos], &name[pos + 1..])),
        None => Err(anyhow!(
            "Binary name {} is not in binary.alternative format",
            name
        )),
    }
}

pub async fn run(binary: Binary) -> Result<()> {
    let mut context = ctf::load()?;
    let challenge_name = match context.path.as_slice() {
        [challenge_name] => challenge_name,
        _ => bail!("Not in a challenge directory"),
    };
    let challenge = ctf::find_challenge_mut(&mut context.ctf, &challenge_name)?;
    match binary.subcmd {
        SubCommand::Show(_) => {
            for binary in &challenge.binaries {
                for alternative in &binary.alternatives {
                    let mut text = style(format!("{}.{}", binary.name, alternative.name));
                    if option::contains(&binary.default_alternative, &alternative.name) {
                        text = text.bold();
                    }
                    println!("{}", text);
                }
            }
        }
        SubCommand::Add(add) => {
            let (binary_name, alternative_name) = split(&add.name)?;
            let path =
                ctf::alternative_path(&context.root, challenge_name, binary_name, alternative_name);
            if !path.exists() {
                bail!("Binary {}.{} does not exist", binary_name, alternative_name);
            }
            if let Some(binary) = ctf::try_find_binary_mut(challenge, binary_name) {
                if ctf::try_find_alternative_mut(binary, alternative_name).is_some() {
                    /* Do nothing. */
                } else {
                    binary.alternatives.push(ctf::BinaryAlternative {
                        name: alternative_name.to_owned(),
                        url: None,
                        checksum: None,
                    });
                }
            } else {
                challenge.binaries.push(ctf::Binary {
                    name: binary_name.to_owned(),
                    alternatives: vec![ctf::BinaryAlternative {
                        name: alternative_name.to_owned(),
                        url: None,
                        checksum: None,
                    }],
                    default_alternative: Some(alternative_name.to_owned()),
                });
                ctf::set_default_alternative(
                    &context.root,
                    &challenge_name,
                    challenge.binaries.last_mut().unwrap(),
                    alternative_name,
                )?;
            }
            git::commit(&context, &format!("Add binary {}", add.name))?;
        }
        SubCommand::Rm(rm) => {
            let (binary_name, alternative_name) = split(&rm.name)?;
            let binary = ctf::find_binary_mut(challenge, binary_name)?;
            let n_alternatives = binary.alternatives.len();
            binary
                .alternatives
                .retain(|alternative| alternative.name != alternative_name);
            if binary.alternatives.len() + 1 != n_alternatives {
                bail!("Binary {} does not exist", rm.name);
            }
            if option::contains(&binary.default_alternative, &alternative_name) {
                binary.default_alternative = None;
                std::fs::remove_file(ctf::default_alternative_path(
                    &context.root,
                    challenge_name,
                    binary_name,
                ))?;
            }
            std::fs::remove_file(ctf::alternative_path(
                &context.root,
                challenge_name,
                binary_name,
                alternative_name,
            ))?;
            if binary.alternatives.is_empty() {
                challenge
                    .binaries
                    .retain(|binary| binary.name != binary_name);
            }
            git::commit(&context, &format!("Remove binary {}", rm.name))?;
        }
        SubCommand::Default(default) => {
            let (binary_name, alternative_name) = split(&default.name)?;
            let challenge_name = challenge.name.to_owned();
            let binary = ctf::find_binary_mut(challenge, binary_name)?;
            ctf::find_alternative_mut(binary, alternative_name)?;
            ctf::set_default_alternative(&context.root, &challenge_name, binary, alternative_name)?;
            git::commit(&context, &format!("Select binary {}", default.name))?;
        }
    }
    Ok(())
}
