use clap::Parser;
use console::style;
use std::fmt::{Display, Formatter};

use anyhow::{anyhow, bail, Result};

use crate::ctf;
use crate::ctf::{resolve_challenge_mut, BinaryAlternative, Challenge, CTF};
use crate::git;
use crate::option;
use crate::path::path_to_str;
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
    /// Names in `binary.alternative` format
    #[clap(required = true)]
    pub names: Vec<String>,
}

#[derive(Parser)]
pub struct Rm {
    /// Names
    #[clap(required = true)]
    pub names: Vec<String>,
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
) -> Result<(&'a mut Challenge, String, PathBuf)> {
    let (challenge, rest) = resolve_challenge_mut(ctf, root, cwd, PathBuf::from(s))?;
    let binary_name = path_to_str(&rest)?.to_string();
    let binary_path = root.join(&challenge.name).join(rest);
    if !binary_path.is_file() {
        bail!("Binary {} does not exist", binary_path.display());
    }
    Ok((challenge, binary_name, binary_path))
}

struct FullBinaryAlternativeName {
    challenge_name: String,
    binary_name: String,
    alternative_name: String,
}

impl Display for FullBinaryAlternativeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}.{}",
            self.challenge_name, self.binary_name, self.alternative_name
        )
    }
}

/// Attempts to add an alternative to an existing binary.
fn try_add_alternative(challenge: &mut Challenge, name: &str) -> Option<FullBinaryAlternativeName> {
    let mut pos = 0;
    while pos < name.len() {
        pos = match name[pos..].find('.') {
            Some(x) => pos + x,
            None => name.len(),
        };
        if let Some(binary) = ctf::try_find_binary_mut(&mut challenge.binaries, &name[..pos]) {
            let alternative_name = if pos == name.len() {
                "orig"
            } else {
                &name[pos + 1..]
            };
            return Some(FullBinaryAlternativeName {
                challenge_name: challenge.name.to_string(),
                binary_name: binary.name.to_string(),
                alternative_name: ctf::try_find_alternative_mut(binary, alternative_name)
                    .map(|alterantive| alterantive.name.to_string())
                    .unwrap_or_else(|| {
                        binary.alternatives.push(ctf::BinaryAlternative {
                            name: alternative_name.to_owned(),
                            url: None,
                            checksum: None,
                        });
                        binary.alternatives.last_mut().unwrap().name.to_string()
                    }),
            });
        }
        pos += 1;
    }
    None
}

fn alternatives_to_string(mut s: String, alternatives: &[FullBinaryAlternativeName]) -> String {
    match alternatives {
        [first] => s.push_str(&first.to_string()),
        [first, second] => {
            s.push_str(&first.to_string());
            s.push_str(" and ");
            s.push_str(&second.to_string());
        }
        _ => {
            for (i, alternative) in alternatives.iter().enumerate() {
                s.push_str(&alternative.to_string());
                if i == alternatives.len() - 2 {
                    s.push_str(", and ")
                } else if i != alternatives.len() - 1 {
                    s.push_str(", ")
                }
            }
        }
    }
    s
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
            let mut alternatives = vec![];
            let mut todo_copy = vec![];
            for name in &add.names {
                let (challenge, name, path) =
                    resolve(&mut context.ctf, &context.root, &context.cwd, name)?;
                alternatives.push(try_add_alternative(challenge, &name).unwrap_or_else(|| {
                    let mut orig = path.as_os_str().to_os_string();
                    orig.push(".orig");
                    todo_copy.push((path, orig));
                    challenge.binaries.push(ctf::Binary {
                        name,
                        alternatives: vec![BinaryAlternative {
                            name: "orig".to_string(),
                            url: None,
                            checksum: None,
                        }],
                        default_alternative: Some("orig".to_string()),
                    });
                    let binary = challenge.binaries.last_mut().unwrap();
                    let alternative = binary.alternatives.last_mut().unwrap();
                    FullBinaryAlternativeName {
                        challenge_name: challenge.name.to_string(),
                        binary_name: binary.name.to_string(),
                        alternative_name: alternative.name.to_string(),
                    }
                }))
            }
            for (src, dst) in todo_copy {
                fs::copy(src, dst)?;
            }
            git::commit(
                &context,
                &alternatives_to_string("Add ".to_string(), &alternatives),
            )?;
        }
        SubCommand::Rm(rm) => {
            let mut alternatives = vec![];
            let mut todo_rm = vec![];
            for name in rm.names {
                let (challenge, s, _) =
                    resolve(&mut context.ctf, &context.root, &context.cwd, &name)?;
                let (binary_name, alternative_name) = split(&s)?;
                let binary = ctf::find_binary_mut(&mut challenge.binaries, binary_name)?;
                let n_alternatives = binary.alternatives.len();
                binary
                    .alternatives
                    .retain(|alternative| alternative.name != alternative_name);
                if binary.alternatives.len() + 1 != n_alternatives {
                    bail!("Binary {} does not exist", name);
                }
                if option::contains(&binary.default_alternative, &alternative_name) {
                    binary.default_alternative = None;
                    fs::remove_file(ctf::default_alternative_path(
                        &context.root,
                        &challenge.name,
                        binary_name,
                    ))?;
                }
                todo_rm.push(ctf::alternative_path(
                    &context.root,
                    &challenge.name,
                    binary_name,
                    alternative_name,
                ));
                if binary.alternatives.is_empty() {
                    challenge
                        .binaries
                        .retain(|binary| binary.name != binary_name);
                }
                alternatives.push(FullBinaryAlternativeName {
                    challenge_name: challenge.name.to_string(),
                    binary_name: binary_name.to_string(),
                    alternative_name: alternative_name.to_string(),
                })
            }
            for path in todo_rm {
                fs::remove_file(path)?;
            }
            git::commit(
                &context,
                &alternatives_to_string("Remove ".to_string(), &alternatives),
            )?;
        }
        SubCommand::Default(default) => {
            let (challenge, s, _) =
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
