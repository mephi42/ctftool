use clap::Parser;
use console::style;

use anyhow::{anyhow, bail, Result};

use crate::ctf;
use crate::ctf::{Challenge, CTF};
use crate::git;
use crate::option;
use crate::path::relativize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub struct Binary {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
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

#[derive(Parser)]
pub struct Show {}

#[derive(Parser)]
pub struct Add {
    /// Name in `binary.alternative` format
    pub name: String,
}

#[derive(Parser)]
pub struct Rm {
    /// Name
    pub name: String,
}

#[derive(Parser)]
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

fn resolve<'a>(
    ctf: &'a mut CTF,
    root: &Path,
    cwd: &Path,
    s: &str,
) -> Result<(&'a mut Challenge, PathBuf, String)> {
    let path = PathBuf::from(s);
    if !path.is_file() {
        bail!("Binary {} does not exist", path.display());
    }
    let (canonical_path, relative_path) = relativize(root, cwd, path)?;
    let os_challenge_name = relative_path
        .components()
        .next()
        .ok_or_else(|| {
            anyhow!(
                "{} is not in a challenge directory",
                canonical_path.display()
            )
        })?
        .as_os_str();
    let challenge_name = os_challenge_name
        .to_str()
        .ok_or_else(|| anyhow!("{:?} contains non-UTF-8 characters", os_challenge_name))?;
    let challenge = ctf::find_challenge_mut(ctf, challenge_name)?;
    let binary_name_path = relative_path.components().skip(1).collect::<PathBuf>();
    let binary_name = binary_name_path
        .to_str()
        .ok_or_else(|| anyhow!("Non-UTF-8 characters in {}", binary_name_path.display()))?;
    Ok((challenge, canonical_path, binary_name.to_string()))
}

pub async fn run(binary: Binary, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    match binary.subcmd {
        SubCommand::Show(_) => {
            let challenge_name = context
                .path
                .first()
                .ok_or_else(|| anyhow!("Not in a challenge directory"))?;
            let challenge = ctf::find_challenge(&context.ctf, challenge_name)?;
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
            let (challenge, p, name) =
                resolve(&mut context.ctf, &context.root, &context.cwd, &add.name)?;
            let mut pos = 0;
            let mut found = false;
            while pos < name.len() {
                pos = match name[pos..].find('.') {
                    Some(x) => pos + x,
                    None => name.len(),
                };
                if let Some(binary) =
                    ctf::try_find_binary_mut(&mut challenge.binaries, &name[..pos])
                {
                    let alternative_name = if pos == name.len() {
                        "orig"
                    } else {
                        &name[pos + 1..]
                    };
                    if ctf::try_find_alternative_mut(binary, alternative_name).is_some() {
                        /* Do nothing. */
                    } else {
                        binary.alternatives.push(ctf::BinaryAlternative {
                            name: alternative_name.to_owned(),
                            url: None,
                            checksum: None,
                        });
                    }
                    found = true;
                    break;
                }
                pos += 1;
            }
            if !found {
                let mut orig = p.as_os_str().to_os_string();
                orig.push(".orig");
                fs::copy(p, orig)?;
                challenge.binaries.push(ctf::Binary {
                    name,
                    alternatives: vec![ctf::BinaryAlternative {
                        name: "orig".to_string(),
                        url: None,
                        checksum: None,
                    }],
                    default_alternative: Some("orig".to_string()),
                });
            }
            git::commit(&context, &format!("Add binary {}", add.name))?;
        }
        SubCommand::Rm(rm) => {
            let (challenge, _, s) =
                resolve(&mut context.ctf, &context.root, &context.cwd, &rm.name)?;
            let (binary_name, alternative_name) = split(&s)?;
            let binary = ctf::find_binary_mut(&mut challenge.binaries, binary_name)?;
            let n_alternatives = binary.alternatives.len();
            binary
                .alternatives
                .retain(|alternative| alternative.name != alternative_name);
            if binary.alternatives.len() + 1 != n_alternatives {
                bail!("Binary {} does not exist", rm.name);
            }
            if option::contains(&binary.default_alternative, &alternative_name) {
                binary.default_alternative = None;
                fs::remove_file(ctf::default_alternative_path(
                    &context.root,
                    &challenge.name,
                    binary_name,
                ))?;
            }
            fs::remove_file(ctf::alternative_path(
                &context.root,
                &challenge.name,
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
            let (challenge, _, s) =
                resolve(&mut context.ctf, &context.root, &context.cwd, &default.name)?;
            let (binary_name, alternative_name) = split(&s)?;
            let challenge_name = challenge.name.to_owned();
            let binary = ctf::find_binary_mut(&mut challenge.binaries, binary_name)?;
            ctf::find_alternative_mut(binary, alternative_name)?;
            ctf::set_default_alternative(&context.root, &challenge_name, binary, alternative_name)?;
            git::commit(&context, &format!("Select binary {}", default.name))?;
        }
    }
    Ok(())
}
