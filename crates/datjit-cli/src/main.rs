use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "datjit",
    version,
    about = "Synthetic data generation from DDL schemas"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic data from a DDL schema
    Generate(commands::generate::GenerateArgs),
    /// Validate a DDL schema without generating data
    Validate(commands::validate::ValidateArgs),
    /// Inspect a DDL schema: print parsed summary, dependency graph, and volume plan
    Inspect(commands::inspect::InspectArgs),
    /// Manage corpus data sources
    Corpus(commands::corpus::CorpusArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => commands::generate::run(args),
        Commands::Validate(args) => commands::validate::run(args),
        Commands::Inspect(args) => commands::inspect::run(args),
        Commands::Corpus(args) => commands::corpus::run(args),
    }
}
