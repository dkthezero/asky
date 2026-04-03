use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "asky",
    about = "Agent skill and instruction manager CLI & TUI",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Remove asky configuration files from the active scope
    Clean {
        /// Recursively clean from global folder instead of workspace folder
        #[arg(short, long)]
        global: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
