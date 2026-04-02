use std::collections::HashMap;

use indexmap::IndexMap;
use serde_yaml::Value as YamlValue;

use datjit_core::error::DatjitError;
use datjit_core::model::*;
use datjit_core::ports::DdlParser;
use datjit_core::types::TypeExpr;

use crate::decorator_parser::{parse_decorators, split_type_and_decorators};
use crate::type_parser::parse_type;

pub struct YamlParser;

impl DdlParser for YamlParser {
    fn parse(&self, input: &str) -> Result<DdlDocument, DatjitError> {
        let yaml: YamlValue =
            serde_yaml::from_str(input).map_err(|e| DatjitError::parse("yaml", e.to_string()))?;

        let mapping = yaml
            .as_mapping()
            .ok_or_else(|| DatjitError::parse("root", "expected YAML mapping at root"))?;

        let domain = get_string(mapping, "domain")
            .ok_or_else(|| DatjitError::parse("root", "missing required field: domain"))?;

        let mut doc = DdlDocument::new(domain);
        doc.version = get_string(mapping, "version");
        doc.seed = get_u64(mapping, "seed");
        doc.locale = get_string(mapping, "locale").unwrap_or_else(|| "en-US".into());

        // Parse volume
        if let Some(vol) = mapping.get(&yaml_key("volume")) {
            doc.volume = parse_volume(vol)?;
        }

        // Parse generation config
        if let Some(gen) = mapping.get(&yaml_key("generation")) {
            doc.generation = parse_generation_config(gen)?;
        }

        // Propagate top-level seed to generation config
        if doc.seed.is_some() && doc.generation.seed.is_none() {
            doc.generation.seed = doc.seed;
        }

        // Parse enums
        if let Some(enums) = mapping.get(&yaml_key("enums")) {
            doc.enums = parse_enums(enums)?;
        }

        // Parse types
        if let Some(types) = mapping.get(&yaml_key("types")) {
            doc.types = parse_types(types)?;
        }

        // Parse entities
        if let Some(entities) = mapping.get(&yaml_key("entities")) {
            doc.entities = parse_entities(entities)?;
        }

        // Parse rules
        if let Some(rules) = mapping.get(&yaml_key("rules")) {
            doc.rules = parse_rules(rules)?;
        }

        // Parse tools
        if let Some(tools) = mapping.get(&yaml_key("tools")) {
            doc.tools = parse_tools(tools)?;
        }

        Ok(doc)
    }
}

fn yaml_key(key: &str) -> YamlValue {
    YamlValue::String(key.into())
}

fn get_string(mapping: &serde_yaml::Mapping, key: &str) -> Option<String> {
    mapping
        .get(&yaml_key(key))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn get_u64(mapping: &serde_yaml::Mapping, key: &str) -> Option<u64> {
    mapping.get(&yaml_key(key)).and_then(|v| v.as_u64())
}

fn parse_volume(value: &YamlValue) -> Result<HashMap<String, VolumeSpec>, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("volume", "expected mapping"))?;

    let mut result = HashMap::new();
    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("volume", "expected string key"))?;

        let spec = if v.is_null() || v.as_str() == Some("~") {
            VolumeSpec::Inferred
        } else if let Some(n) = v.as_u64() {
            VolumeSpec::Exact(n as usize)
        } else if let Some(s) = v.as_str() {
            if let Some((lo, hi)) = s.split_once("..") {
                let lo: usize = lo.trim().parse().map_err(|_| {
                    DatjitError::parse("volume", format!("invalid range lo: {lo}"))
                })?;
                let hi: usize = hi.trim().parse().map_err(|_| {
                    DatjitError::parse("volume", format!("invalid range hi: {hi}"))
                })?;
                VolumeSpec::Range(lo, hi)
            } else {
                let n: usize = s.parse().map_err(|_| {
                    DatjitError::parse("volume", format!("invalid volume: {s}"))
                })?;
                VolumeSpec::Exact(n)
            }
        } else {
            return Err(DatjitError::parse(
                "volume",
                format!("invalid volume spec for {name}"),
            ));
        };

        result.insert(name.to_string(), spec);
    }
    Ok(result)
}

fn parse_generation_config(value: &YamlValue) -> Result<GenerationConfig, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("generation", "expected mapping"))?;

    let mut cfg = GenerationConfig::default();

    if let Some(seed) = get_u64(mapping, "seed") {
        cfg.seed = Some(seed);
    }
    if let Some(locale) = get_string(mapping, "locale") {
        cfg.locale = locale;
    }
    if let Some(ns) = get_string(mapping, "null_strategy") {
        cfg.null_strategy = match ns.as_str() {
            "realistic" => NullStrategy::Realistic,
            "never" => NullStrategy::Never,
            "sparse" => NullStrategy::Sparse,
            _ => {
                return Err(DatjitError::parse(
                    "generation",
                    format!("unknown null_strategy: {ns}"),
                ))
            }
        };
    }
    if let Some(fmt) = get_string(mapping, "id_format") {
        cfg.id_format = match fmt.as_str() {
            "uuid" => IdFormat::Uuid,
            "sequential" => IdFormat::Sequential,
            "cuid" => IdFormat::Cuid,
            "ulid" => IdFormat::Ulid,
            _ => {
                return Err(DatjitError::parse(
                    "generation",
                    format!("unknown id_format: {fmt}"),
                ))
            }
        };
    }
    if let Some(df) = get_string(mapping, "date_format") {
        cfg.date_format = df;
    }
    if let Some(cf) = get_string(mapping, "currency_format") {
        cfg.currency_format = match cf.as_str() {
            "decimal" => CurrencyFormat::Decimal,
            "integer_cents" => CurrencyFormat::IntegerCents,
            _ => {
                return Err(DatjitError::parse(
                    "generation",
                    format!("unknown currency_format: {cf}"),
                ))
            }
        };
    }

    Ok(cfg)
}

fn parse_enums(value: &YamlValue) -> Result<HashMap<String, EnumDef>, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("enums", "expected mapping"))?;

    let mut result = HashMap::new();
    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("enums", "expected string key"))?;

        let variants = if let Some(seq) = v.as_sequence() {
            parse_enum_variants(seq)?
        } else {
            return Err(DatjitError::parse(
                "enums",
                format!("expected sequence for enum {name}"),
            ));
        };

        result.insert(
            name.to_string(),
            EnumDef {
                name: name.to_string(),
                variants,
            },
        );
    }
    Ok(result)
}

fn parse_enum_variants(seq: &[YamlValue]) -> Result<Vec<EnumVariant>, DatjitError> {
    let mut variants = Vec::new();
    for item in seq {
        if let Some(s) = item.as_str() {
            variants.push(EnumVariant::simple(s));
        } else if let Some(mapping) = item.as_mapping() {
            let value = get_string(mapping, "value")
                .ok_or_else(|| DatjitError::parse("enum", "weighted variant missing 'value'"))?;
            let label = get_string(mapping, "label");
            let weight = mapping
                .get(&yaml_key("weight"))
                .and_then(|v| v.as_f64());
            variants.push(EnumVariant {
                value,
                label,
                weight,
            });
        } else {
            return Err(DatjitError::parse("enum", "invalid variant format"));
        }
    }
    Ok(variants)
}

fn parse_types(value: &YamlValue) -> Result<HashMap<String, TypeDef>, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("types", "expected mapping"))?;

    let mut result = HashMap::new();
    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("types", "expected string key"))?;

        let fields_mapping = v
            .as_mapping()
            .ok_or_else(|| DatjitError::parse("types", format!("expected mapping for type {name}")))?;

        let fields = parse_fields(fields_mapping)?;

        result.insert(
            name.to_string(),
            TypeDef {
                name: name.to_string(),
                fields,
            },
        );
    }
    Ok(result)
}

fn parse_entities(value: &YamlValue) -> Result<IndexMap<String, Entity>, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("entities", "expected mapping"))?;

    let mut result = IndexMap::new();
    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("entities", "expected string key"))?;

        let entity_mapping = v.as_mapping().ok_or_else(|| {
            DatjitError::parse("entities", format!("expected mapping for entity {name}"))
        })?;

        let mut entity = Entity::new(name);

        // Parse _meta (entity-level decorators)
        if let Some(meta_val) = entity_mapping.get(&yaml_key("_meta")) {
            if let Some(meta_str) = meta_val.as_str() {
                let (_, dec_strs) = split_type_and_decorators(&format!("_ {meta_str}"));
                entity.meta = parse_decorators(&dec_strs)?;
            }
        }

        // Parse _coherence
        if let Some(coh_val) = entity_mapping.get(&yaml_key("_coherence")) {
            if let Some(coh_mapping) = coh_val.as_mapping() {
                for (gk, gv) in coh_mapping {
                    let group_name = gk.as_str().unwrap_or("").to_string();
                    let fields: Vec<String> = gv
                        .as_sequence()
                        .map(|seq| {
                            seq.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    entity.coherence_groups.insert(group_name, fields);
                }
            }
        }

        // Parse fields (skip _meta and _coherence)
        let fields = parse_fields_excluding(entity_mapping, &["_meta", "_coherence"])?;
        entity.fields = fields;

        result.insert(name.to_string(), entity);
    }
    Ok(result)
}

fn parse_fields(mapping: &serde_yaml::Mapping) -> Result<IndexMap<String, Field>, DatjitError> {
    parse_fields_excluding(mapping, &[])
}

fn parse_fields_excluding(
    mapping: &serde_yaml::Mapping,
    exclude: &[&str],
) -> Result<IndexMap<String, Field>, DatjitError> {
    let mut fields = IndexMap::new();

    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("field", "expected string key"))?;

        if exclude.contains(&name) {
            continue;
        }

        let field_str = v
            .as_str()
            .ok_or_else(|| {
                DatjitError::parse("field", format!("expected string value for field {name}"))
            })?;

        let (type_str, decorator_strs) = split_type_and_decorators(field_str);

        // Handle nullable shorthand: type ends with ? before decorators
        let (type_str, extra_nullable) = if type_str.ends_with('?')
            && !type_str.starts_with("->")
            && !type_str.starts_with("<->")
        {
            (type_str[..type_str.len() - 1].to_string(), true)
        } else {
            (type_str, false)
        };

        let type_expr = parse_type(&type_str)
            .map_err(|e| DatjitError::parse(format!("field.{name}"), e.to_string()))?;

        // Wrap in Nullable if trailing ?
        let type_expr = if extra_nullable {
            TypeExpr::Compound(datjit_core::types::CompoundType::Nullable(Box::new(
                type_expr,
            )))
        } else {
            type_expr
        };

        let decorators = parse_decorators(&decorator_strs)
            .map_err(|e| DatjitError::parse(format!("field.{name}"), e.to_string()))?;

        fields.insert(
            name.to_string(),
            Field {
                name: name.to_string(),
                type_expr,
                decorators,
            },
        );
    }

    Ok(fields)
}

fn parse_rules(value: &YamlValue) -> Result<Vec<Rule>, DatjitError> {
    let seq = value
        .as_sequence()
        .ok_or_else(|| DatjitError::parse("rules", "expected sequence"))?;

    let mut rules = Vec::new();
    for item in seq {
        if let Some(s) = item.as_str() {
            // For now, store rules as simple comparison expressions
            // Full rule parsing will be enhanced in Phase 5
            let rule = parse_rule_string(s)?;
            rules.push(rule);
        }
    }
    Ok(rules)
}

fn parse_rule_string(input: &str) -> Result<Rule, DatjitError> {
    let input = input.trim();

    // Extract modifier
    let (expr_str, modifier) = if input.contains("@probability(") {
        let parts: Vec<&str> = input.splitn(2, "@probability(").collect();
        let prob_str = parts[1].trim_end_matches(')');
        let prob: f64 = prob_str
            .parse()
            .map_err(|_| DatjitError::parse("rule", format!("invalid probability: {prob_str}")))?;
        (parts[0].trim(), RuleModifier::Probability(prob))
    } else if input.contains("@strict") {
        let parts: Vec<&str> = input.splitn(2, "@strict").collect();
        (parts[0].trim(), RuleModifier::Strict)
    } else if input.contains("@warn") {
        let parts: Vec<&str> = input.splitn(2, "@warn").collect();
        (parts[0].trim(), RuleModifier::Warn)
    } else {
        (input, RuleModifier::default())
    };

    // Parse the expression
    let expression = if expr_str.starts_with("if ") {
        // Conditional: if X then Y
        let rest = &expr_str[3..];
        if let Some(then_idx) = rest.find(" then ") {
            let condition_str = &rest[..then_idx];
            let then_str = &rest[then_idx + 6..];
            RuleExpression::Conditional {
                condition: Box::new(parse_comparison(condition_str)?),
                then: Box::new(parse_comparison(then_str)?),
            }
        } else {
            return Err(DatjitError::parse("rule", "conditional missing 'then'"));
        }
    } else if expr_str.starts_with("unique(") {
        // Unique composite
        let inner = &expr_str[7..expr_str.len() - 1];
        let paths: Vec<FieldPath> = inner
            .split(',')
            .map(|s| FieldPath::parse(s.trim()))
            .collect();
        RuleExpression::UniqueComposite(paths)
    } else if expr_str.starts_with("count(") {
        // Count constraint
        let close_paren = expr_str.find(')').unwrap_or(expr_str.len());
        let path_str = &expr_str[6..close_paren];
        let rest = &expr_str[close_paren + 1..].trim();
        let (op, value) = parse_op_and_operand(rest)?;
        RuleExpression::CountConstraint {
            path: FieldPath::parse(path_str),
            op,
            value,
        }
    } else {
        parse_comparison(expr_str)?
    };

    Ok(Rule {
        expression,
        modifier,
    })
}

fn parse_comparison(input: &str) -> Result<RuleExpression, DatjitError> {
    let input = input.trim();

    // Try to find comparison operator
    let ops = ["==", "!=", ">=", "<=", ">", "<"];
    for op_str in ops {
        if let Some(idx) = input.find(op_str) {
            let left = input[..idx].trim();
            let right = input[idx + op_str.len()..].trim();

            let op = match op_str {
                "==" => CompOp::Eq,
                "!=" => CompOp::Neq,
                ">=" => CompOp::Gte,
                "<=" => CompOp::Lte,
                ">" => CompOp::Gt,
                "<" => CompOp::Lt,
                _ => unreachable!(),
            };

            let right_operand = parse_rule_operand(right);

            return Ok(RuleExpression::Comparison {
                left: FieldPath::parse(left),
                op,
                right: right_operand,
            });
        }
    }

    Err(DatjitError::parse(
        "rule",
        format!("cannot parse rule expression: {input}"),
    ))
}

fn parse_op_and_operand(input: &str) -> Result<(CompOp, RuleOperand), DatjitError> {
    let input = input.trim();
    let ops = [">=", "<=", "==", "!=", ">", "<", "in"];
    for op_str in ops {
        if input.starts_with(op_str) {
            let rest = input[op_str.len()..].trim();
            let op = match op_str {
                "==" => CompOp::Eq,
                "!=" => CompOp::Neq,
                ">=" => CompOp::Gte,
                "<=" => CompOp::Lte,
                ">" => CompOp::Gt,
                "<" => CompOp::Lt,
                "in" => CompOp::In,
                _ => unreachable!(),
            };
            return Ok((op, parse_rule_operand(rest)));
        }
    }
    Err(DatjitError::parse(
        "rule",
        format!("cannot parse operator in: {input}"),
    ))
}

fn parse_rule_operand(input: &str) -> RuleOperand {
    let input = input.trim();
    if input == "null" {
        return RuleOperand::Null;
    }
    if input == "true" {
        return RuleOperand::Bool(true);
    }
    if input == "false" {
        return RuleOperand::Bool(false);
    }
    if let Ok(n) = input.parse::<i64>() {
        return RuleOperand::Int(n);
    }
    if let Ok(n) = input.parse::<f64>() {
        return RuleOperand::Float(n);
    }
    if input.starts_with('"') && input.ends_with('"') {
        return RuleOperand::String(input[1..input.len() - 1].to_string());
    }
    if input.contains("..") {
        let parts: Vec<&str> = input.split("..").collect();
        if let (Ok(lo), Ok(hi)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
            return RuleOperand::Range(lo, hi);
        }
    }
    // Assume it's a field path
    RuleOperand::FieldPath(FieldPath::parse(input))
}

fn parse_tools(value: &YamlValue) -> Result<HashMap<String, ToolOverrides>, DatjitError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("tools", "expected mapping"))?;

    let mut result = HashMap::new();
    for (k, v) in mapping {
        let name = k
            .as_str()
            .ok_or_else(|| DatjitError::parse("tools", "expected string key"))?;

        let tool_mapping = v.as_mapping().ok_or_else(|| {
            DatjitError::parse("tools", format!("expected mapping for {name}"))
        })?;

        let overrides = ToolOverrides {
            list: parse_list_override(tool_mapping)?,
            create: parse_mutation_override(tool_mapping, "create")?,
            update: parse_mutation_override(tool_mapping, "update")?,
            delete: parse_delete_override(tool_mapping)?,
        };

        result.insert(name.to_string(), overrides);
    }
    Ok(result)
}

fn parse_list_override(
    mapping: &serde_yaml::Mapping,
) -> Result<Option<ListOverride>, DatjitError> {
    let list_val = match mapping.get(&yaml_key("list")) {
        Some(v) => v,
        None => return Ok(None),
    };

    let list_mapping = list_val
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("tools", "expected mapping for list"))?;

    Ok(Some(ListOverride {
        filters: list_mapping
            .get(&yaml_key("filters"))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        sorts: list_mapping
            .get(&yaml_key("sorts"))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        page_size: list_mapping
            .get(&yaml_key("page_size"))
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
        max_page_size: list_mapping
            .get(&yaml_key("max_page_size"))
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
    }))
}

fn parse_mutation_override(
    mapping: &serde_yaml::Mapping,
    key: &str,
) -> Result<Option<MutationOverride>, DatjitError> {
    let val = match mapping.get(&yaml_key(key)) {
        Some(v) => v,
        None => return Ok(None),
    };

    // Check for "disabled"
    if val.as_str() == Some("disabled") {
        return Ok(Some(MutationOverride {
            disabled: true,
            required: None,
            optional: None,
            mutable: None,
            immutable: None,
            defaults: None,
        }));
    }

    let m = val
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("tools", format!("expected mapping for {key}")))?;

    fn get_string_vec(m: &serde_yaml::Mapping, key: &str) -> Option<Vec<String>> {
        m.get(&YamlValue::String(key.into()))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
    }

    Ok(Some(MutationOverride {
        disabled: false,
        required: get_string_vec(m, "required"),
        optional: get_string_vec(m, "optional"),
        mutable: get_string_vec(m, "mutable"),
        immutable: get_string_vec(m, "immutable"),
        defaults: m
            .get(&yaml_key("defaults"))
            .and_then(|v| v.as_mapping())
            .map(|dm| {
                dm.iter()
                    .filter_map(|(k, v)| {
                        Some((k.as_str()?.to_string(), v.as_str()?.to_string()))
                    })
                    .collect()
            }),
    }))
}

fn parse_delete_override(
    mapping: &serde_yaml::Mapping,
) -> Result<Option<DeleteOverride>, DatjitError> {
    let val = match mapping.get(&yaml_key("delete")) {
        Some(v) => v,
        None => return Ok(None),
    };

    if val.as_str() == Some("disabled") {
        return Ok(Some(DeleteOverride {
            disabled: true,
            strategy: None,
        }));
    }

    let m = val
        .as_mapping()
        .ok_or_else(|| DatjitError::parse("tools", "expected mapping for delete"))?;

    let strategy = get_string(m, "strategy").map(|s| match s.as_str() {
        "soft" => DeleteStrategy::Soft,
        "hard" => DeleteStrategy::Hard,
        _ => DeleteStrategy::Hard,
    });

    Ok(Some(DeleteOverride {
        disabled: false,
        strategy,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_document() {
        let yaml = r#"
domain: test
entities:
  User:
    id: uuid @primary
    name: string
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        assert_eq!(doc.domain, "test");
        assert_eq!(doc.locale, "en-US");
        assert_eq!(doc.entities.len(), 1);

        let user = &doc.entities["User"];
        assert_eq!(user.name, "User");
        assert_eq!(user.fields.len(), 2);
        assert!(user.fields["id"].is_primary());
    }

    #[test]
    fn test_full_document() {
        let yaml = r#"
domain: project_management
version: 0.1.0
seed: 42
locale: en-US

volume:
  User: 200
  Project: 50

enums:
  Priority: [critical, high, medium, low]
  TaskStatus: [backlog, todo, in_progress, review, done, cancelled]

entities:
  User:
    _meta: "@timestamps @soft_delete"
    id: uuid @primary
    name: person.full
    email: email @unique
    role: enum(admin, manager, member, viewer) @dist(5, 15, 70, 10)

  Project:
    id: uuid @primary
    name: string @len(3..60) @searchable
    lead: ->User @filterable
    status: enum(planning, active, paused, completed, archived) @dist(10, 50, 10, 20, 10)

rules:
  - Task.assignee.org == Task.project.org @strict
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();

        assert_eq!(doc.domain, "project_management");
        assert_eq!(doc.version, Some("0.1.0".into()));
        assert_eq!(doc.seed, Some(42));
        assert_eq!(doc.entities.len(), 2);

        // Check volume
        assert!(matches!(doc.volume["User"], VolumeSpec::Exact(200)));
        assert!(matches!(doc.volume["Project"], VolumeSpec::Exact(50)));

        // Check enums
        assert_eq!(doc.enums["Priority"].variants.len(), 4);
        assert_eq!(doc.enums["TaskStatus"].variants.len(), 6);

        // Check User entity
        let user = &doc.entities["User"];
        assert_eq!(user.fields.len(), 4);
        assert!(user.fields["email"].is_unique());

        // Check Project reference
        let project = &doc.entities["Project"];
        let lead = &project.fields["lead"];
        assert!(matches!(
            lead.type_expr,
            datjit_core::types::TypeExpr::Reference(
                datjit_core::types::ReferenceType::BelongsTo { .. }
            )
        ));

        // Check rules
        assert_eq!(doc.rules.len(), 1);
    }

    #[test]
    fn test_enums_weighted() {
        let yaml = r#"
domain: test
enums:
  Continent:
    - value: NA
      label: "North America"
      weight: 25
    - value: EU
      label: "Europe"
      weight: 30
entities:
  Item:
    id: uuid @primary
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        let continent = &doc.enums["Continent"];
        assert_eq!(continent.variants.len(), 2);
        assert_eq!(continent.variants[0].weight, Some(25.0));
        assert_eq!(continent.variants[1].label, Some("Europe".into()));
    }

    #[test]
    fn test_volume_range() {
        let yaml = r#"
domain: test
volume:
  Order: "4000..6000"
  LineItem: ~
entities:
  Order:
    id: uuid @primary
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        assert!(matches!(doc.volume["Order"], VolumeSpec::Range(4000, 6000)));
        assert!(matches!(doc.volume["LineItem"], VolumeSpec::Inferred));
    }

    #[test]
    fn test_missing_domain() {
        let yaml = r#"
entities:
  User:
    id: uuid @primary
"#;
        let parser = YamlParser;
        assert!(parser.parse(yaml).is_err());
    }

    #[test]
    fn test_types_section() {
        let yaml = r#"
domain: test
types:
  Address:
    line1: address.street
    city: address.city
    zip: address.zip
entities:
  Customer:
    id: uuid @primary
    billing_address: Address
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        assert!(doc.types.contains_key("Address"));
        assert_eq!(doc.types["Address"].fields.len(), 3);

        let billing = &doc.entities["Customer"].fields["billing_address"];
        assert!(matches!(billing.type_expr, datjit_core::types::TypeExpr::Named(ref s) if s == "Address"));
    }

    #[test]
    fn test_tools_section() {
        let yaml = r#"
domain: test
entities:
  Task:
    id: uuid @primary
tools:
  Task:
    list:
      filters: [project, status]
      sorts: [created_at]
      page_size: 50
    update:
      mutable: [title, status]
    delete: disabled
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        let task_tools = &doc.tools["Task"];

        let list = task_tools.list.as_ref().unwrap();
        assert_eq!(list.page_size, Some(50));
        assert_eq!(list.filters.as_ref().unwrap().len(), 2);

        let update = task_tools.update.as_ref().unwrap();
        assert!(!update.disabled);
        assert_eq!(update.mutable.as_ref().unwrap().len(), 2);

        let delete = task_tools.delete.as_ref().unwrap();
        assert!(delete.disabled);
    }

    #[test]
    fn test_coherence_groups() {
        let yaml = r#"
domain: test
entities:
  Employee:
    _coherence:
      identity: [first_name, last_name, email]
      location: [office, timezone]
    id: uuid @primary
    first_name: person.first
    last_name: person.last
    email: email
    office: string
    timezone: timezone
"#;
        let parser = YamlParser;
        let doc = parser.parse(yaml).unwrap();
        let emp = &doc.entities["Employee"];
        assert_eq!(emp.coherence_groups.len(), 2);
        assert_eq!(emp.coherence_groups["identity"].len(), 3);
        assert_eq!(emp.coherence_groups["location"].len(), 2);
    }
}
