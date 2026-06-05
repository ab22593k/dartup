mod cache;
mod cli;
mod config;
mod environment;
mod install;
mod project;
mod releases;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure directories exist on startup
    std::fs::create_dir_all(config::envs_dir())?;
    std::fs::create_dir_all(config::engine_cache_dir())?;
    std::fs::create_dir_all(config::git_cache_dir())?;

    match cli.command {
        Commands::Install { version, force } => install::install_version(&version, force),
        Commands::Use { version, global } => {
            if global {
                environment::set_global(&version)
            } else {
                project::set_project_version(&version)
            }
        }
        Commands::List => environment::list_versions(),
        Commands::Current => environment::show_current(),
        Commands::Remove { version } => environment::remove_version(&version),
        Commands::Releases { all } => releases::list_releases(all),
        Commands::Gc => cache::run_gc(),
        Commands::Doctor => environment::run_doctor(),
    }
}
