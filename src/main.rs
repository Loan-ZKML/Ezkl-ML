use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Mandatory address to operate on
    address: String,

    /// Sets a custom config file
    #[arg(short, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, action = clap::ArgAction::Count)]
    debug: u8,

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

fn main() {
    let cli = Cli::parse();

    println!("Address: {}", cli.address);

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }

    // Change the address comparison:
    let is_medium_tier = *address == MEDIUM_TIER_ADDRESS;

    // Change the script creation by removing the extra argument:
    let mut script = String::from(r#"#!/usr/bin/env bash
// ...existing script content...
"#);
}
