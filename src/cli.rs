use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dartup", about = "A fast Flutter version manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a specific Flutter version (e.g., "3.29.0", "stable", "beta")
    Install {
        version: String,
        /// Re-download even if cached
        #[arg(short, long)]
        force: bool,
    },
    /// Use a version in the current project (creates .dartup.json)
    Use {
        version: String,
        /// Set as global default instead of project-local
        #[arg(short)]
        global: bool,
    },
    /// List installed Flutter versions
    List,
    /// Show currently active Flutter version
    Current,
    /// Remove an installed version
    Remove { version: String },
    /// List available Flutter releases from the official channel
    Releases {
        /// Show all releases (not just recent)
        #[arg(long)]
        all: bool,
    },
    /// Run garbage collection on unused cached artifacts
    Gc,
    /// Check that dartup is set up correctly
    Doctor,
}
