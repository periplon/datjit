use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use datjit_core::ports::{DataGenerator, DdlParser, OutputWriter};
use datjit_generator::GenerationEngine;
use datjit_output::{CsvWriter, JsonWriter, NdJsonWriter, SqlDialect, SqlWriter, YamlWriter};
use datjit_parser::YamlParser;

#[derive(Args)]
pub struct GenerateArgs {
    /// Path to DDL schema file
    pub schema: PathBuf,

    /// Output file or directory (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format: json, csv, ndjson, yaml, sql
    #[arg(short, long, default_value = "json")]
    pub format: String,

    /// Override seed for deterministic generation
    #[arg(long)]
    pub seed: Option<u64>,

    /// Override locale
    #[arg(long)]
    pub locale: Option<String>,

    /// Pretty-print output
    #[arg(long)]
    pub pretty: bool,

    /// Override volume per entity (comma-separated, e.g. "User=100,Order=500")
    #[arg(long)]
    pub volume: Option<String>,

    /// Generate only this entity (and its dependencies)
    #[arg(long)]
    pub entity: Option<String>,

    /// SQL dialect: postgres, mysql, sqlite
    #[arg(long, default_value = "postgres")]
    pub sql_dialect: String,

    /// Show generation plan without generating data
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: GenerateArgs) -> Result<()> {
    let input = fs::read_to_string(&args.schema)
        .with_context(|| format!("Failed to read schema: {}", args.schema.display()))?;

    let parser = YamlParser;
    let mut doc = parser
        .parse(&input)
        .with_context(|| "Failed to parse DDL schema")?;

    // Apply --volume overrides
    if let Some(volume_str) = &args.volume {
        for pair in volume_str.split(',') {
            let pair = pair.trim();
            if let Some((entity, count_str)) = pair.split_once('=') {
                let entity = entity.trim();
                let count: usize = count_str
                    .trim()
                    .parse()
                    .with_context(|| format!("Invalid volume count for '{entity}': {count_str}"))?;
                doc.volume.insert(
                    entity.to_string(),
                    datjit_core::model::VolumeSpec::Exact(count),
                );
            } else {
                anyhow::bail!("Invalid --volume format: '{pair}'. Expected Entity=N");
            }
        }
    }

    // Apply --entity filter: remove entities that are not the target or its dependencies
    if let Some(target_entity) = &args.entity {
        if !doc.entities.contains_key(target_entity.as_str()) {
            anyhow::bail!("Entity '{}' not found in schema", target_entity);
        }
        let deps = collect_entity_deps(&doc, target_entity);
        let to_remove: Vec<String> = doc
            .entities
            .keys()
            .filter(|k| !deps.contains(k.as_str()))
            .cloned()
            .collect();
        for name in to_remove {
            doc.entities.shift_remove(&name);
            doc.volume.remove(&name);
        }
    }

    // Handle --dry-run
    if args.dry_run {
        eprintln!("Dry run - generation plan:");
        eprintln!("  Domain: {}", doc.domain);
        eprintln!("  Entities ({})", doc.entities.len());

        // Build dependency info
        for (name, entity) in &doc.entities {
            let vol = doc
                .volume
                .get(name)
                .map(|v| match v {
                    datjit_core::model::VolumeSpec::Exact(n) => format!("{n}"),
                    datjit_core::model::VolumeSpec::Range(lo, hi) => format!("{lo}..{hi}"),
                    datjit_core::model::VolumeSpec::Inferred => "~inferred".into(),
                })
                .unwrap_or_else(|| "100 (default)".into());

            let deps: Vec<String> = entity
                .fields
                .values()
                .filter_map(|f| match &f.type_expr {
                    datjit_core::types::TypeExpr::Reference(
                        datjit_core::types::ReferenceType::BelongsTo { target, .. },
                    ) => Some(target.clone()),
                    datjit_core::types::TypeExpr::Reference(
                        datjit_core::types::ReferenceType::ManyToMany { target },
                    ) => Some(target.clone()),
                    _ => None,
                })
                .collect();

            let dep_str = if deps.is_empty() {
                String::new()
            } else {
                format!(" -> depends on: {}", deps.join(", "))
            };

            eprintln!(
                "    {name}: {} fields, volume={vol}{dep_str}",
                entity.fields.len()
            );
        }

        if !doc.rules.is_empty() {
            eprintln!("  Rules: {}", doc.rules.len());
        }
        return Ok(());
    }

    let mut engine = GenerationEngine::new();
    if let Some(seed) = args.seed {
        engine = engine.with_seed(seed);
    }
    if let Some(locale) = &args.locale {
        engine = engine.with_locale(locale.clone());
    }

    let dataset = engine
        .generate(&doc)
        .with_context(|| "Failed to generate data")?;

    // Select writer based on format
    let sql_dialect = SqlDialect::from_str(&args.sql_dialect).unwrap_or(SqlDialect::Postgres);

    let writer: Box<dyn OutputWriter> = match args.format.to_lowercase().as_str() {
        "json" => Box::new(JsonWriter::new(args.pretty || args.output.is_none())),
        "csv" => Box::new(CsvWriter::new()),
        "ndjson" | "jsonl" => Box::new(NdJsonWriter::new()),
        "yaml" | "yml" => Box::new(YamlWriter::new()),
        "sql" => Box::new(SqlWriter::new(sql_dialect)),
        other => {
            anyhow::bail!("Unsupported format: {other}. Supported: json, csv, ndjson, yaml, sql");
        }
    };

    // Write output
    match args.output {
        Some(path) => {
            let mut file = fs::File::create(&path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            writer
                .write(&dataset, &mut file)
                .with_context(|| "Failed to write output")?;
            eprintln!(
                "Generated {} total rows across {} entities -> {}",
                dataset.total_rows(),
                dataset.entities.len(),
                path.display()
            );
        }
        None => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writer
                .write(&dataset, &mut handle)
                .with_context(|| "Failed to write to stdout")?;
        }
    }

    Ok(())
}

/// Collect an entity and all its transitive dependencies.
fn collect_entity_deps(
    doc: &datjit_core::model::DdlDocument,
    entity_name: &str,
) -> std::collections::HashSet<String> {
    let mut deps = std::collections::HashSet::new();
    let mut queue = vec![entity_name.to_string()];

    while let Some(name) = queue.pop() {
        if !deps.insert(name.clone()) {
            continue;
        }
        if let Some(entity) = doc.entities.get(&name) {
            for field in entity.fields.values() {
                match &field.type_expr {
                    datjit_core::types::TypeExpr::Reference(
                        datjit_core::types::ReferenceType::BelongsTo { target, .. },
                    ) => {
                        queue.push(target.clone());
                    }
                    datjit_core::types::TypeExpr::Reference(
                        datjit_core::types::ReferenceType::ManyToMany { target },
                    ) => {
                        queue.push(target.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    deps
}
