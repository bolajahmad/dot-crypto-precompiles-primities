//! PVM Precompiles CLI
//!
//! A developer utility for interacting with cryptographic precompiles
//! implemented for the PVM runtime.
//!
//! # Structure
//!
//! - `commands/` - Command handlers for each cryptographic primitive
//!   - `schnorr` - Schnorr signature operations
//!   - (future: `bls` - BLS12-381 operations)
//! - `utils` - Shared utility functions
//!
//! # Usage
//!
//! ```bash
//! pvmcli schnorr sign -s <SECRET_KEY> -m "message"
//! pvmcli schnorr verify -p <PUBKEY> -n <NONCE> -s <SIG> -m "message"
//! pvmcli schnorr test-data --secret-key <SECRET_KEY> --nonce <NONCE_SEED>
//! pvmcli bls random-g1
//! pvmcli bls g2-add --point-a <POINT> --point-b <POINT>
//! pvmcli bls g1-msm-testdata --pairs 3 --output both
//! pvmcli bls g2-msm --data <MSM_DATA>
//! pvmcli bls map-fp-to-g1 --fp <FP_INPUT>
//! pvmcli bls sign --secret-key <SK> --message "hello"
//! pvmcli bls batch-sign-testdata --count 4 --output both
//! pvmcli bls batch-smoke --count 4 --output summary
//! ```

use clap::{Parser, Subcommand};

mod commands;
mod utils;

use commands::{bls::BlsCommands, schnorr::SchnorrCommands};

/// PVM Precompiles CLI - Developer utility for cryptographic precompiles.
#[derive(Parser)]
#[command(name = "pvmcli")]
#[command(about = "PVM Precompiles CLI - Developer utility for cryptographic precompiles")]
#[command(version = "0.1.0")]
#[command(author = "Jamal Jones")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Top-level commands organized by cryptographic primitive.
#[derive(Subcommand)]
enum Commands {
    /// BLS12-381 point generation and addition utilities
    Bls {
        #[command(subcommand)]
        action: BlsCommands,
    },

    /// Schnorr signature operations (sign, verify, test-data)
    Schnorr {
        #[command(subcommand)]
        action: SchnorrCommands,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bls { action } => commands::bls::handle(action),
        Commands::Schnorr { action } => commands::schnorr::handle(action),
    }
}
