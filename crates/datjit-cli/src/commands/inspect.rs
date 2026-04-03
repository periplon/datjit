use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use datjit_core::model::tool_inference;
use datjit_core::ports::DdlParser;
use datjit_core::types::{ReferenceType, TypeExpr};
use datjit_parser::YamlParser;

#[derive(Args)]
pub struct InspectArgs {
    /// Path to DDL schema file
    pub schema: PathBuf,

    /// Infer and display CRUD tools for each entity
    #[arg(long)]
    pub infer_tools: bool,
}

pub fn run(args: InspectArgs) -> Result<()> {
    let input = fs::read_to_string(&args.schema)
        .with_context(|| format!("Failed to read schema: {}", args.schema.display()))?;

    let parser = YamlParser;
    let doc = parser
        .parse(&input)
        .with_context(|| "Failed to parse DDL schema")?;

    println!("Schema Summary");
    println!("==============");
    println!("Domain: {}", doc.domain);
    if let Some(version) = &doc.version {
        println!("Version: {version}");
    }
    println!("Locale: {}", doc.locale);
    if let Some(seed) = doc.seed {
        println!("Seed: {seed}");
    }
    println!();

    // Entities
    println!("Entities ({}):", doc.entities.len());
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

        println!("  {name}: {} fields, volume={vol}", entity.fields.len());

        for (field_name, field) in &entity.fields {
            let type_str = format!("{:?}", field.type_expr);
            let decorators: Vec<String> = field
                .decorators
                .iter()
                .map(|d| format!("{d:?}"))
                .collect();
            let dec_str = if decorators.is_empty() {
                String::new()
            } else {
                format!(" [{}]", decorators.join(", "))
            };
            let meta = match (&field.label, &field.description) {
                (Some(l), Some(d)) => format!("  — \"{l}\": {d}"),
                (Some(l), None) => format!("  — \"{l}\""),
                (None, Some(d)) => format!("  — {d}"),
                (None, None) => String::new(),
            };
            println!("    {field_name}: {type_str}{dec_str}{meta}");
        }
    }
    println!();

    // Dependency graph
    println!("Dependency Graph:");
    for (name, entity) in &doc.entities {
        let deps: Vec<String> = entity
            .fields
            .values()
            .filter_map(|f| match &f.type_expr {
                TypeExpr::Reference(ReferenceType::BelongsTo { target, .. }) => {
                    Some(target.clone())
                }
                TypeExpr::Reference(ReferenceType::ManyToMany { target }) => {
                    Some(target.clone())
                }
                _ => None,
            })
            .collect();

        if deps.is_empty() {
            println!("  {name} (no dependencies)");
        } else {
            println!("  {name} -> {}", deps.join(", "));
        }
    }
    println!();

    // Volume plan
    println!("Volume Plan:");
    for name in doc.entities.keys() {
        let vol = doc
            .volume
            .get(name)
            .map(|v| match v {
                datjit_core::model::VolumeSpec::Exact(n) => format!("{n} rows"),
                datjit_core::model::VolumeSpec::Range(lo, hi) => format!("{lo}..{hi} rows"),
                datjit_core::model::VolumeSpec::Inferred => "~inferred".into(),
            })
            .unwrap_or_else(|| "100 rows (default)".into());
        println!("  {name}: {vol}");
    }

    if !doc.enums.is_empty() {
        println!();
        println!("Enums ({}):", doc.enums.len());
        for (name, enum_def) in &doc.enums {
            let has_descriptions = enum_def
                .variants
                .iter()
                .any(|v| v.description.is_some());
            if has_descriptions {
                println!("  {name}: {} variants", enum_def.variants.len());
                for v in &enum_def.variants {
                    if let Some(desc) = &v.description {
                        let label = v
                            .label
                            .as_deref()
                            .map(|l| format!(" ({l})"))
                            .unwrap_or_default();
                        println!("    - {}{label}: {desc}", v.value);
                    } else {
                        println!("    - {}", v.value);
                    }
                }
            } else {
                println!("  {name}: {} variants", enum_def.variants.len());
            }
        }
    }

    if !doc.rules.is_empty() {
        println!();
        println!("Rules ({}):", doc.rules.len());
        for (i, rule) in doc.rules.iter().enumerate() {
            println!("  [{}] {:?} ({:?})", i + 1, rule.expression, rule.modifier);
        }
    }

    if args.infer_tools {
        println!();
        println!("Inferred Tools");
        println!("==============");
        for (_name, entity) in &doc.entities {
            let tools = tool_inference::infer_tools(entity);
            println!();
            println!("  {}:", tools.entity_name);

            if let Some(list) = &tools.list {
                println!("    LIST (page_size={})", list.page_size);
                if !list.filters.is_empty() {
                    println!("      filters: {}", list.filters.join(", "));
                }
                if !list.sorts.is_empty() {
                    println!("      sorts: {}", list.sorts.join(", "));
                }
                if !list.search_fields.is_empty() {
                    println!("      search: {}", list.search_fields.join(", "));
                }
            }

            if let Some(get) = &tools.get {
                println!("    GET (by {})", get.primary_key);
            }

            if let Some(create) = &tools.create {
                println!("    CREATE");
                if !create.required_fields.is_empty() {
                    println!("      required: {}", create.required_fields.join(", "));
                }
                if !create.optional_fields.is_empty() {
                    println!("      optional: {}", create.optional_fields.join(", "));
                }
            }

            if let Some(update) = &tools.update {
                println!("    UPDATE");
                if !update.mutable_fields.is_empty() {
                    println!("      mutable: {}", update.mutable_fields.join(", "));
                }
            }

            if let Some(delete) = &tools.delete {
                println!("    DELETE ({})", delete.strategy);
            }
        }
    }

    Ok(())
}
