use anyhow::{bail, Result};
use clap::Parser;

use crate::ctf::{find_service_mut, resolve_challenge_mut, try_find_service_mut};
use crate::option;
use crate::{ctf, git};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Service {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Prints a list of services and their URLs
    #[clap(name = "show")]
    Show(Show),

    /// Adds a new service
    #[clap(name = "add")]
    Add(Add),

    /// Sets a service URL
    #[clap(name = "set-url")]
    SetUrl(SetUrl),

    /// Removes an existing service
    #[clap(name = "rm")]
    Rm(Rm),
}

#[derive(Parser)]
pub struct Show {}

#[derive(Parser)]
pub struct Add {
    /// Service name
    pub name: String,

    /// Service URL
    pub url: String,
}

#[derive(Parser)]
pub struct SetUrl {
    /// Service name
    pub name: String,

    /// Service URL
    pub url: String,
}

#[derive(Parser)]
pub struct Rm {
    /// Service name
    pub name: String,
}

pub fn run(service: Service, current_dir: PathBuf) -> Result<()> {
    let mut context = ctf::load(current_dir)?;
    let (challenge, _) = resolve_challenge_mut(
        &mut context.ctf,
        &context.root,
        &context.cwd,
        PathBuf::new(),
    )?;
    match service.subcmd {
        SubCommand::Show(_show) => {
            for service in &challenge.services {
                println!(
                    "{} - {}",
                    &service.name.as_deref().unwrap_or("<none>"),
                    service.url
                )
            }
        }
        SubCommand::Add(add) => {
            let existing = try_find_service_mut(challenge, &add.name);
            if existing.is_some() {
                bail!("Service {} already exists", &add.name);
            }
            let message = format!("Add service {} to challenge {}", &add.name, &challenge.name);
            challenge.services.push(ctf::Service {
                name: Some(add.name),
                url: add.url,
            });
            git::commit(&context, &message)?;
        }
        SubCommand::SetUrl(set_url) => {
            let message = format!(
                "Set service {} URL to {} in challenge {}",
                set_url.name, set_url.url, challenge.name
            );
            let service = find_service_mut(challenge, &set_url.name)?;
            service.url = set_url.url;
            git::commit(&context, &message)?;
        }
        SubCommand::Rm(rm) => {
            let message = format!(
                "Remove service {} from challenge {}",
                &rm.name, &challenge.name
            );
            let n_services = challenge.services.len();
            challenge
                .services
                .retain(|service| !option::contains(&service.name, &rm.name));
            if challenge.services.len() + 1 != n_services {
                bail!("Service {} does not exist", &rm.name);
            }
            git::commit(&context, &message)?;
        }
    }
    Ok(())
}
