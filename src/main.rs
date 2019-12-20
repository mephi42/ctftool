use anyhow::Result;
use clap::Clap;

pub mod commands;
pub mod ctf;
pub mod engines;
pub mod git;
pub mod subprocess;

/// Automates all the boring CTF stuff
#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Init(init) => commands::init::run(init),
        SubCommand::Remote(remote) => commands::remote::run(remote),
        SubCommand::Fetch(fetch) => commands::fetch::run(fetch).await,
    }
}
