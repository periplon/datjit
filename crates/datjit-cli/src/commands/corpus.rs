use anyhow::Result;
use clap::{Args, Subcommand};

use datjit_corpus::updater;

#[derive(Args)]
pub struct CorpusArgs {
    #[command(subcommand)]
    pub command: CorpusCommand,
}

#[derive(Subcommand)]
pub enum CorpusCommand {
    /// List known corpus sources and their categories
    List,
    /// Show corpus directory location and installed data
    Info,
    /// Print download instructions for corpus data (or download if updater feature is enabled)
    Update,
}

pub fn run(args: CorpusArgs) -> Result<()> {
    match args.command {
        CorpusCommand::List => run_list(),
        CorpusCommand::Info => run_info(),
        CorpusCommand::Update => run_update(),
    }
}

fn run_list() -> Result<()> {
    let sources = updater::known_sources();

    println!("Known Corpus Sources");
    println!("====================");
    println!();

    for source in &sources {
        println!("  {} [{}]", source.name, source.category);
        println!("    {}", source.description);
        println!("    License: {}", source.license);
        println!("    URL: {}", source.url);
        println!();
    }

    println!("{} sources available", sources.len());

    Ok(())
}

fn run_info() -> Result<()> {
    let status = updater::check_corpus_status()
        .map_err(|e| anyhow::anyhow!("Failed to check corpus status: {e}"))?;

    println!("Corpus Info");
    println!("===========");
    println!();
    println!("Directory: {}", status.corpus_dir.display());
    println!();

    if status.installed_locales.is_empty() && status.installed_files.is_empty() {
        println!("No corpus data installed.");
        println!();
        println!("Run `datjit corpus update` for download instructions.");
    } else {
        if !status.installed_locales.is_empty() {
            println!(
                "Installed locales ({}): {}",
                status.installed_locales.len(),
                status.installed_locales.join(", ")
            );
        }

        if !status.installed_files.is_empty() {
            println!();
            println!("Installed files ({}):", status.installed_files.len());
            for (name, size) in &status.installed_files {
                println!("  {} ({} bytes)", name, size);
            }
        }

        println!();
        println!("Total size: {} bytes", status.total_size_bytes);
    }

    Ok(())
}

fn run_update() -> Result<()> {
    let corpus_dir = updater::default_corpus_dir();
    let sources = updater::known_sources();

    println!("Corpus Update");
    println!("=============");
    println!();
    println!(
        "Target directory: {}",
        corpus_dir.display()
    );
    println!();
    println!(
        "To install corpus data, download the following sources and place them in the corpus directory:"
    );
    println!();

    for source in &sources {
        println!("  {} [{}]", source.name, source.category);
        println!("    {}", source.url);
        println!();
    }

    println!("You can create the corpus directory with:");
    println!("  mkdir -p {}", corpus_dir.display());
    println!();
    println!("Tip: Enable the `updater` feature for automatic downloads:");
    println!("  cargo install datjit --features updater");

    Ok(())
}
