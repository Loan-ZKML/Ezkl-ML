use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Mandatory address to operate on
    address: String,

    /// Sets a custom config file
    #[arg(short, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Test {
        #[arg(short, long)]
        list: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Address: {}", cli.address);

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    if let Some(Commands::Test { list }) = cli.command {
        if list {
            // Here you would implement actual test listing functionality
            println!("Test command executed with list flag enabled");
        }
    }

    Ok(())
}
