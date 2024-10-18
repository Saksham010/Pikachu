use clap::{Parser, Subcommand}; // Import `clap`

mod trusted_setup;
mod prover;
mod verifier;

/// CLI interface for running different parts of the zkSNARK system
#[derive(Parser)]
#[command(name = "Pikachu", version = "1.0", about = "Run Pikachu setup, prover, or verifier")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run trusted setup
    TrustedSetup,
    /// Run prover
    Prover,
    /// Run verifier with the provided proof
    Verifier {
        /// Base64-encoded proof string
        proof: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::TrustedSetup => trusted_setup::main(),
        Commands::Prover => prover::main(),
        Commands::Verifier { proof } => verifier::main(&proof),
    }
}
