use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "keyway")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Hide,
    Quit,
    Reload,
}
