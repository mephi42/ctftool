use anyhow::{anyhow, bail, Result};
use clap::Parser;

use crate::{ctf, distro, git, subprocess};
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
    let yml = "docker-compose.yml";
    match docker.subcmd {
        SubCommand::Init(_init) => {
            let mut packages_variants = vec![];
            for binary in &challenge.binaries {
                println!("Analyzing {}...", binary.name);
                let binary_path = challenge_dir.join(&binary.name);
                if let Some(packages) = distro::get_packages(&binary_path)? {
                    println!("  Arch: {}", packages.arch.unwrap_or("?"));
                    println!(
                        "  Distro: {} {}",
                        packages.distro.unwrap_or("?"),
                        packages.distro_version.unwrap_or("?")
                    );
                    println!(
                        "  Libc: {}",
                        &packages.libc_version.as_deref().unwrap_or("?")
                    );
                    packages_variants.push(packages);
                }
            }
            let packages = distro::merge_packages_variants(packages_variants);
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
                serde_yaml::Value::String("arch".into()),
                serde_yaml::Value::String(packages.arch.unwrap_or(distro::DEFAULT_ARCH).into()),
            );
            args.insert(
                serde_yaml::Value::String("distro".into()),
                serde_yaml::Value::String(format!(
                    "{}:{}",
                    packages.distro.unwrap_or(distro::DEFAULT_DISTRO),
                    packages
                        .distro_version
                        .unwrap_or(distro::DEFAULT_DISTRO_VERSION)
                )),
            );
            args.insert(
                serde_yaml::Value::String("libc_version".into()),
                serde_yaml::Value::String(
                    packages
                        .libc_version
                        .unwrap_or(distro::DEFAULT_LIBC_VERSION.into()),
                ),
            );
            build.insert(
                serde_yaml::Value::String("args".into()),
                serde_yaml::Value::Mapping(args),
            );
            fs::write(challenge_dir.join(yml), serde_yaml::to_string(&compose)?)?;
            git::commit(
                &context,
                &format!("Add Dockerfile and {} for {}", yml, challenge_name),
            )?;
        }
        SubCommand::Exec(exec) => {
            subprocess::check_call(Command::new("xhost").args(["+local:root"]))?;
            subprocess::check_call(
                Command::new("docker")
                    .args([
                        "compose",
                        &format!("--file={}", yml),
                        "up",
                        "--build",
                        "--detach",
                    ])
                    .current_dir(&challenge_dir),
            )?;
            subprocess::check_call(
                Command::new("docker")
                    .args(
                        [
                            "compose".to_string(),
                            format!("--file={}", yml),
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
                .args(["compose", &format!("--file={}", yml), "down"])
                .current_dir(&challenge_dir),
        )?,
        SubCommand::Rmi(_rmi) => subprocess::check_call(
            Command::new("docker")
                .args(["compose", &format!("--file={}", yml), "down", "--rmi=local"])
                .current_dir(&challenge_dir),
        )?,
    }
    Ok(())
}
