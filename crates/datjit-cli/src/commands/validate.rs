use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use datjit_core::ports::DdlParser;
use datjit_parser::YamlParser;

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to DDL schema file
    pub schema: PathBuf,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    let input = fs::read_to_string(&args.schema)
        .with_context(|| format!("Failed to read schema: {}", args.schema.display()))?;

    let parser = YamlParser;
    match parser.parse(&input) {
        Ok(doc) => {
            eprintln!("Schema is valid.");
            eprintln!("  Domain: {}", doc.domain);
            if let Some(version) = &doc.version {
                eprintln!("  Version: {version}");
            }
            eprintln!("  Entities: {}", doc.entities.len());
            for (name, entity) in &doc.entities {
                eprintln!("    {name}: {} fields", entity.fields.len());
            }
            if !doc.enums.is_empty() {
                eprintln!("  Enums: {}", doc.enums.len());
            }
            if !doc.types.is_empty() {
                eprintln!("  Types: {}", doc.types.len());
            }
            if !doc.rules.is_empty() {
                eprintln!("  Rules: {}", doc.rules.len());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Schema validation failed:");
            eprintln!("  {e}");
            std::process::exit(1);
        }
    }
}
