use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Once;

use anyhow::Result;
use clap::Parser;

pub mod commands;
pub mod ctf;
pub mod distro;
pub mod engines;
pub mod git;
pub mod http;
pub mod option;
pub mod path;
pub mod subprocess;

/// Automates all the boring CTF stuff
#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Initializes a git repository for a single CTF
    #[clap(name = "init")]
    Init(commands::init::Init),

    /// Manages links to CTF websites
    #[clap(name = "remote")]
    Remote(commands::remote::Remote),

    /// Downloads challenge metadata
    #[clap(name = "fetch")]
    Fetch(commands::fetch::Fetch),

    /// Downloads challenge binaries
    #[clap(name = "checkout")]
    Checkout(commands::checkout::Checkout),

    /// Logs into a CTF
    #[clap(name = "login")]
    Login(commands::login::Login),

    /// Manages challenge binaries
    #[clap(name = "binary")]
    Binary(commands::binary::Binary),

    /// Manages challenges
    #[clap(name = "challenge")]
    Challenge(commands::challenge::Challenge),

    /// Manages Docker container
    #[clap(name = "docker")]
    Docker(commands::docker::Docker),
}

pub async fn main<I, T>(args: I, current_dir: PathBuf) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let opts: Opts = Opts::try_parse_from(args)?;
    match opts.subcmd {
        SubCommand::Init(init) => commands::init::run(init, current_dir),
        SubCommand::Remote(remote) => commands::remote::run(remote, current_dir),
        SubCommand::Fetch(fetch) => commands::fetch::run(fetch, current_dir).await,
        SubCommand::Checkout(checkout) => commands::checkout::run(checkout, current_dir).await,
        SubCommand::Login(login) => commands::login::run(login, current_dir).await,
        SubCommand::Binary(binary) => commands::binary::run(binary, current_dir).await,
        SubCommand::Challenge(challenge) => commands::challenge::run(challenge, current_dir),
        SubCommand::Docker(docker) => commands::docker::run(docker, current_dir).await,
    }
}

#[tokio::main]
pub async fn main_sync<I, T>(args: I, current_dir: PathBuf) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    main(args, current_dir).await
}

static INIT_LOGGING: Once = Once::new();

pub fn init_logging() {
    INIT_LOGGING.call_once(env_logger::init);
}
