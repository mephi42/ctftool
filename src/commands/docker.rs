use anyhow::{anyhow, bail, Result};
use clap::Parser;

use crate::distro::Packages;
use crate::{ctf, distro, git, http, subprocess};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
pub struct Docker {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Generates images/Dockerfile and docker-compose.yml
    #[clap(name = "init")]
    Init(Init),

    /// Runs a command inside a Docker container
    #[clap(name = "exec")]
    Exec(Exec),

    /// Removes the Docker container
    #[clap(name = "rm")]
    Rm(Rm),

    /// Removes the Docker image
    #[clap(name = "rmi")]
    Rmi(Rmi),
}

#[derive(Parser)]
pub struct Init {}

#[derive(Parser)]
pub struct Exec {
    pub command: String,

    /// Arguments
    pub argv: Vec<String>,
}

#[derive(Parser)]
pub struct Rm {}

#[derive(Parser)]
pub struct Rmi {}

fn get_mapping<'a>(
    mapping: &'a mut serde_yaml::Mapping,
    key: &str,
) -> Result<&'a mut serde_yaml::Mapping> {
    mapping
        .get_mut(&serde_yaml::Value::String(key.into()))
        .ok_or_else(|| anyhow!("Missing \"{}\"", key))?
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("\"{}\" is not a mapping", key))
}

pub async fn run(docker: Docker, current_dir: PathBuf) -> Result<()> {
    let context = ctf::load(current_dir)?;
    let challenge_name = match context.path.as_slice() {
        [challenge_name, ..] => challenge_name,
        _ => bail!("Not in a challenge directory"),
    };
    let challenge = ctf::find_challenge(&context.ctf, challenge_name)?;
    let challenge_dir = context.root.join(challenge_name);
    match docker.subcmd {
        SubCommand::Init(_init) => {
            let client = http::mk_client(&[])?;
            let mut maybe_packages = None;
            for binary in &challenge.binaries {
                let binary_path = challenge_dir.join(&binary.name);
                if let Some(packages) = distro::get_packages(&client, &binary_path).await? {
                    maybe_packages = Some(packages);
                    break;
                }
            }
            let packages = maybe_packages.unwrap_or_else(Packages::default);
            let image = challenge_dir.join("image");
            fs::create_dir_all(&image)?;
            fs::write(
                image.join("Dockerfile"),
                include_str!("docker/image/Dockerfile").as_bytes(),
            )?;
            let mut compose: serde_yaml::Mapping =
                serde_yaml::from_str(include_str!("docker/docker-compose.yml"))?;
            let services = get_mapping(&mut compose, "services")?;
            let main = get_mapping(services, "main")?;
            let build = get_mapping(main, "build")?;
            let mut args = serde_yaml::Mapping::new();
            args.insert(
                serde_yaml::Value::String("ubuntu_version".into()),
                serde_yaml::Value::String(packages.distro_version),
            );
            args.insert(
                serde_yaml::Value::String("libc_version".into()),
                serde_yaml::Value::String(packages.libc_version),
            );
            build.insert(
                serde_yaml::Value::String("args".into()),
                serde_yaml::Value::Mapping(args),
            );
            fs::write(
                challenge_dir.join("docker-compose.yml"),
                serde_yaml::to_string(&compose)?,
            )?;
            git::commit(
                &context,
                &format!(
                    "Add Dockerfile and docker-compose.yml for {}",
                    challenge_name
                ),
            )?;
        }
        SubCommand::Exec(exec) => {
            subprocess::check_call(Command::new("xhost").args(["+local:root"]))?;
            subprocess::check_call(
                Command::new("docker")
                    .args(["compose", "up", "--build", "--detach"])
                    .current_dir(&challenge_dir),
            )?;
            subprocess::check_call(
                Command::new("docker")
                    .args(
                        [
                            "compose".to_string(),
                            "exec".to_string(),
                            "main".to_string(),
                            exec.command,
                        ]
                        .into_iter()
                        .chain(exec.argv.into_iter())
                        .collect::<Vec<String>>(),
                    )
                    .current_dir(&challenge_dir),
            )?;
        }
        SubCommand::Rm(_rm) => subprocess::check_call(
            Command::new("docker")
                .args(["compose", "down"])
                .current_dir(&challenge_dir),
        )?,
        SubCommand::Rmi(_rmi) => subprocess::check_call(
            Command::new("docker")
                .args(["compose", "down", "--rmi=local"])
                .current_dir(&challenge_dir),
        )?,
    }
    Ok(())
}
