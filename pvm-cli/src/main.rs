// crates/pvm-cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pvm-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    BlsGenG1,
    SchnorrSign,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::BlsGenG1 => {
            println!("Generated random G1 point: 0x1234...");
        }
        Commands::SchnorrSign => {
            println!("Generated Schnorr signature: 0xabcd...");
        }
    }
}