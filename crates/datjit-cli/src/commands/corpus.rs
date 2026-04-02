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
    /// Download and install corpus data from remote sources
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
        println!("Run `datjit corpus update` to download corpus data.");
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
                if *size > 1024 * 1024 {
                    println!("  {} ({:.1} MB)", name, *size as f64 / 1024.0 / 1024.0);
                } else {
                    println!("  {} ({:.1} KB)", name, *size as f64 / 1024.0);
                }
            }
        }

        println!();
        if status.total_size_bytes > 1024 * 1024 {
            println!(
                "Total size: {:.1} MB",
                status.total_size_bytes as f64 / 1024.0 / 1024.0
            );
        } else {
            println!(
                "Total size: {:.1} KB",
                status.total_size_bytes as f64 / 1024.0
            );
        }
    }

    Ok(())
}

fn run_update() -> Result<()> {
    let corpus_dir = updater::default_corpus_dir();

    println!("Corpus Update");
    println!("=============");
    println!();
    println!("Target directory: {}", corpus_dir.display());
    println!();

    let report = updater::update_corpus(&corpus_dir, &|msg| {
        println!("{msg}");
    })
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!();
    println!("Summary:");
    println!(
        "  {} files updated ({:.1} MB)",
        report.files_updated.len(),
        report.total_size_bytes as f64 / 1024.0 / 1024.0
    );

    if !report.files_failed.is_empty() {
        println!("  {} sources failed:", report.files_failed.len());
        for (name, err) in &report.files_failed {
            println!("    {name}: {err}");
        }
    }

    Ok(())
}
