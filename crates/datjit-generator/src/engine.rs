use indexmap::IndexMap;

use datjit_core::error::DatjitError;
use datjit_core::model::decorator::Decorator;
use datjit_core::model::DdlDocument;
use datjit_core::ports::generator::{DataGenerator, EntityData, GeneratedDataSet};
use datjit_core::types::{EnumRef, TypeExpr};
use datjit_core::value::Value;

use crate::coherence::generate_coherence_groups;
use crate::constraint::{enforce_cross_row_rules, enforce_rules};
use crate::context::GenerationContext;
use crate::decorator_apply::apply_decorators;
use crate::derived_gen::evaluate_derived;
use crate::field_gen::generate_field;
use crate::plan::GenerationPlan;

pub struct GenerationEngine {
    seed: Option<u64>,
    locale: String,
}

impl GenerationEngine {
    pub fn new() -> Self {
        Self {
            seed: None,
            locale: "en-US".into(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_locale(mut self, locale: String) -> Self {
        self.locale = locale;
        self
    }
}

impl Default for GenerationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DataGenerator for GenerationEngine {
    fn generate(&self, doc: &DdlDocument) -> Result<GeneratedDataSet, DatjitError> {
        let seed = self.seed.or(doc.seed).or(doc.generation.seed);
        let locale = if self.locale != "en-US" {
            self.locale.clone()
        } else {
            doc.locale.clone()
        };

        // Resolve TypeExpr::Named references against doc.enums
        let mut resolved_doc = doc.clone();
        resolve_named_types(&mut resolved_doc);
        let doc = &resolved_doc;

        let mut ctx = GenerationContext::new(seed, locale);
        let plan = GenerationPlan::from_document(doc)?;
        let mut dataset = GeneratedDataSet::new();

        for entity_name in &plan.entity_order {
            let entity = &doc.entities[entity_name];
            let volume = plan.volumes.get(entity_name).copied().unwrap_or(100);

            // Check if entity has @timestamps decorator
            let has_timestamps = entity
                .meta
                .iter()
                .any(|d| matches!(d, Decorator::Timestamps));

            // Determine column names (all non-meta fields + timestamp fields if applicable)
            let mut columns: Vec<String> = entity.fields.keys().cloned().collect();
            if has_timestamps {
                if !columns.contains(&"created_at".to_string()) {
                    columns.push("created_at".into());
                }
                if !columns.contains(&"updated_at".to_string()) {
                    columns.push("updated_at".into());
                }
            }

            let mut entity_data = EntityData::new(entity_name.clone(), columns);

            for _ in 0..volume {
                let mut row = IndexMap::new();
                let mut rule_attempts = 0;
                const MAX_RULE_RETRIES: usize = 10;

                loop {
                    row.clear();

                    // Step 1: Generate coherence groups (including implicit @from fields)
                    let coherence_values = generate_coherence_groups(entity, &mut ctx)?;
                    let coherence_field_set: std::collections::HashSet<&String> =
                        coherence_values.keys().collect();
                    for (k, v) in &coherence_values {
                        row.insert(k.clone(), v.clone());
                    }

                    // Step 2: Generate non-derived, non-coherence fields
                    for (field_name, field) in &entity.fields {
                        // Skip fields already populated by coherence groups
                        if coherence_field_set.contains(field_name) && row.contains_key(field_name)
                        {
                            continue;
                        }

                        // Skip derived, computed, and default_chain fields (evaluated later)
                        if field.is_derived() || field.is_computed() || field.is_default_chain() {
                            row.insert(field_name.clone(), Value::Null);
                            continue;
                        }

                        let value = if field.is_primary() {
                            generate_primary_key(entity_name, &mut ctx)
                        } else if field.is_auto() {
                            generate_auto_field(field_name, &mut ctx)
                        } else {
                            let val = generate_field(field, entity_name, &row, &mut ctx)?;
                            // Apply decorators to non-primary, non-auto fields
                            apply_decorators(val, field, &mut ctx.rng)?
                        };

                        // Enforce uniqueness
                        let value = if field.is_unique() {
                            let mut val = value;
                            let mut attempts = 0;
                            while !ctx.check_and_insert_unique(entity_name, field_name, &val) {
                                attempts += 1;
                                if attempts > 100 {
                                    return Err(DatjitError::UniquenessExhausted {
                                        entity: entity_name.clone(),
                                        field: field_name.clone(),
                                        attempts,
                                    });
                                }
                                val = if field.is_primary() {
                                    generate_primary_key(entity_name, &mut ctx)
                                } else {
                                    let v = generate_field(field, entity_name, &row, &mut ctx)?;
                                    apply_decorators(v, field, &mut ctx.rng)?
                                };
                            }
                            val
                        } else {
                            value
                        };

                        row.insert(field_name.clone(), value);
                    }

                    // Step 2.5: Enforce @dependent_required — if a field with this
                    // decorator is non-null, ensure all dependent fields are also non-null
                    for (field_name, field) in &entity.fields {
                        for dec in &field.decorators {
                            if let Decorator::DependentRequired(deps) = dec {
                                let is_present =
                                    row.get(field_name).map(|v| !v.is_null()).unwrap_or(false);
                                if is_present {
                                    for dep_name in deps {
                                        if let Some(val) = row.get(dep_name) {
                                            if val.is_null() {
                                                // Regenerate the dependent field as non-null
                                                if let Some(dep_field) = entity.fields.get(dep_name)
                                                {
                                                    let new_val = generate_field(
                                                        dep_field,
                                                        entity_name,
                                                        &row,
                                                        &mut ctx,
                                                    )?;
                                                    let new_val = apply_decorators(
                                                        new_val,
                                                        dep_field,
                                                        &mut ctx.rng,
                                                    )?;
                                                    // If still null, generate primitive fallback
                                                    let new_val = if new_val.is_null() {
                                                        Value::String("required".into())
                                                    } else {
                                                        new_val
                                                    };
                                                    row.insert(dep_name.clone(), new_val);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Step 2.6: Evaluate @default_chain fields
                    for (field_name, field) in &entity.fields {
                        if !field.is_default_chain() {
                            continue;
                        }
                        for dec in &field.decorators {
                            if let Decorator::DefaultChain {
                                sources,
                                when,
                                fallback,
                            } = dec
                            {
                                // Check `when` condition
                                let should_eval = match when {
                                    Some(cond_expr) => {
                                        let cond =
                                            evaluate_derived(cond_expr, &row, &ctx.generated)?;
                                        !cond.is_null() && cond != Value::Bool(false)
                                    }
                                    None => true,
                                };

                                if should_eval {
                                    let mut resolved = None;
                                    for source in sources {
                                        let val =
                                            resolve_chain_source(source, &row, &ctx.generated);
                                        if !val.is_null() {
                                            resolved = Some(val);
                                            break;
                                        }
                                    }
                                    let val = resolved.unwrap_or_else(|| {
                                        fallback
                                            .as_ref()
                                            .map(|fb| {
                                                evaluate_derived(fb, &row, &ctx.generated)
                                                    .unwrap_or(Value::Null)
                                            })
                                            .unwrap_or(Value::Null)
                                    });
                                    row.insert(field_name.clone(), val);
                                }
                                break;
                            }
                        }
                    }

                    // Step 2.7: Evaluate @compute fields
                    for (field_name, field) in &entity.fields {
                        if !field.is_computed() {
                            continue;
                        }
                        for dec in &field.decorators {
                            if let Decorator::Compute(branches) = dec {
                                let mut matched = false;
                                for branch in branches {
                                    if let Some(when_expr) = &branch.when {
                                        let cond =
                                            evaluate_derived(when_expr, &row, &ctx.generated)?;
                                        if !cond.is_null() && cond != Value::Bool(false) {
                                            let val = evaluate_derived(
                                                &branch.value,
                                                &row,
                                                &ctx.generated,
                                            )?;
                                            row.insert(field_name.clone(), val);
                                            matched = true;
                                            break;
                                        }
                                    } else {
                                        // else branch
                                        let val =
                                            evaluate_derived(&branch.value, &row, &ctx.generated)?;
                                        row.insert(field_name.clone(), val);
                                        matched = true;
                                        break;
                                    }
                                }
                                if !matched {
                                    row.insert(field_name.clone(), Value::Null);
                                }
                                break;
                            }
                        }
                    }

                    // Step 3: Evaluate @derived fields
                    for (field_name, field) in &entity.fields {
                        if !field.is_derived() {
                            continue;
                        }
                        for dec in &field.decorators {
                            if let Decorator::Derived(expr) = dec {
                                let derived_val = evaluate_derived(expr, &row, &ctx.generated)?;
                                row.insert(field_name.clone(), derived_val);
                                break;
                            }
                        }
                    }

                    // Step 4: Add @timestamps fields if applicable
                    if has_timestamps {
                        let now = "2025-01-15T10:30:00".to_string();
                        row.entry("created_at".into())
                            .or_insert_with(|| Value::DateTime(now.clone()));
                        row.entry("updated_at".into())
                            .or_insert_with(|| Value::DateTime(now));
                    }

                    // Enforce rules after generating the row
                    let rule_result =
                        enforce_rules(&doc.rules, entity_name, &row, &ctx.generated, &mut ctx.rng);

                    match rule_result {
                        Ok(()) => break,
                        Err(_) if rule_attempts < MAX_RULE_RETRIES => {
                            rule_attempts += 1;
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }

                entity_data.rows.push(row.clone());

                // Store in context for reference resolution
                ctx.generated
                    .entry(entity_name.clone())
                    .or_default()
                    .push(row);
            }

            // Step 6: Cross-row validation post-pass
            enforce_cross_row_rules(
                &doc.rules,
                entity_name,
                &mut entity_data.rows,
                &mut ctx.generated,
                &mut ctx.rng,
            );

            dataset.entities.insert(entity_name.clone(), entity_data);
        }

        Ok(dataset)
    }
}

/// Resolve `TypeExpr::Named` references against the document's enum definitions.
/// Converts e.g. `TypeExpr::Named("TaskStatus")` → `TypeExpr::Enum(EnumRef::Inline(variants))`.
/// Resolve a default_chain source field path against the current row and all generated data.
/// Supports both simple field refs ("field") and reference traversal ("ref.field").
fn resolve_chain_source(
    path: &datjit_core::model::decorator::FieldPath,
    row: &IndexMap<String, Value>,
    all_data: &IndexMap<String, Vec<IndexMap<String, Value>>>,
) -> Value {
    if path.segments.is_empty() {
        return Value::Null;
    }

    if path.segments.len() == 1 {
        return row.get(&path.segments[0]).cloned().unwrap_or(Value::Null);
    }

    // Multi-segment: e.g., "wo.gl_acct" — follow reference
    let ref_field = &path.segments[0];
    let target_field = &path.segments[1];

    match row.get(ref_field) {
        Some(Value::Ref(entity_name, pk_value)) => {
            if let Some(entity_rows) = all_data.get(entity_name.as_str()) {
                for entity_row in entity_rows {
                    if let Some(first_val) = entity_row.values().next() {
                        if first_val == pk_value.as_ref() {
                            return entity_row.get(target_field).cloned().unwrap_or(Value::Null);
                        }
                    }
                }
            }
            Value::Null
        }
        _ => Value::Null,
    }
}

fn resolve_named_types(doc: &mut DdlDocument) {
    let enum_variants: std::collections::HashMap<String, Vec<String>> = doc
        .enums
        .iter()
        .map(|(name, def)| {
            (
                name.clone(),
                def.variants.iter().map(|v| v.value.clone()).collect(),
            )
        })
        .collect();

    for entity in doc.entities.values_mut() {
        for field in entity.fields.values_mut() {
            if let TypeExpr::Named(name) = &field.type_expr {
                if let Some(variants) = enum_variants.get(name) {
                    field.type_expr = TypeExpr::Enum(EnumRef::Inline(variants.clone()));
                }
            }
        }
    }
}

fn generate_primary_key(_entity_name: &str, _ctx: &mut GenerationContext) -> Value {
    Value::Uuid(uuid::Uuid::new_v4().to_string())
}

fn generate_auto_field(field_name: &str, _ctx: &mut GenerationContext) -> Value {
    match field_name {
        "created_at" | "updated_at" => Value::DateTime("2025-01-15T10:30:00".into()),
        "version" => Value::Int(1),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::model::decorator::Decorator;
    use datjit_core::model::{Entity, Field};
    use datjit_core::types::{PrimitiveType, SemanticType, TypeExpr};

    fn make_simple_doc() -> DdlDocument {
        let mut doc = DdlDocument::new("test");
        doc.seed = Some(42);

        let mut user = Entity::new("User");
        user.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        user.fields.insert(
            "name".into(),
            Field::new(
                "name",
                TypeExpr::Semantic(SemanticType::new("person", "full")),
            ),
        );
        user.fields.insert(
            "email".into(),
            Field::new("email", TypeExpr::Semantic(SemanticType::new("email", "")))
                .with_decorators(vec![Decorator::Unique]),
        );

        doc.entities.insert("User".into(), user);
        doc.volume
            .insert("User".into(), datjit_core::model::VolumeSpec::Exact(10));
        doc
    }

    #[test]
    fn test_basic_generation() {
        let doc = make_simple_doc();
        let engine = GenerationEngine::new();
        let dataset = engine.generate(&doc).unwrap();

        assert_eq!(dataset.entities.len(), 1);
        let users = &dataset.entities["User"];
        assert_eq!(users.row_count(), 10);
        assert_eq!(users.columns, vec!["id", "name", "email"]);

        // Check all rows have all fields
        for row in &users.rows {
            assert!(row.contains_key("id"));
            assert!(row.contains_key("name"));
            assert!(row.contains_key("email"));
        }
    }

    #[test]
    fn test_unique_emails() {
        let doc = make_simple_doc();
        let engine = GenerationEngine::new();
        let dataset = engine.generate(&doc).unwrap();
        let users = &dataset.entities["User"];

        let emails: Vec<_> = users
            .rows
            .iter()
            .map(|r| r["email"].to_output_string())
            .collect();

        let unique_count = emails
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, emails.len(), "emails should be unique");
    }

    #[test]
    fn test_deterministic() {
        let doc = make_simple_doc();
        let engine = GenerationEngine::new().with_seed(42);
        let dataset1 = engine.generate(&doc).unwrap();
        let dataset2 = engine.generate(&doc).unwrap();

        let names1: Vec<_> = dataset1.entities["User"]
            .rows
            .iter()
            .map(|r| r["name"].to_output_string())
            .collect();
        let names2: Vec<_> = dataset2.entities["User"]
            .rows
            .iter()
            .map(|r| r["name"].to_output_string())
            .collect();

        assert_eq!(names1, names2);
    }

    #[test]
    fn test_reference_generation() {
        let mut doc = DdlDocument::new("test");
        doc.seed = Some(42);

        let mut user = Entity::new("User");
        user.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        doc.entities.insert("User".into(), user);
        doc.volume
            .insert("User".into(), datjit_core::model::VolumeSpec::Exact(5));

        let mut order = Entity::new("Order");
        order.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        order.fields.insert(
            "user".into(),
            Field::new(
                "user",
                TypeExpr::Reference(datjit_core::types::ReferenceType::BelongsTo {
                    target: "User".into(),
                    optional: false,
                }),
            ),
        );
        doc.entities.insert("Order".into(), order);
        doc.volume
            .insert("Order".into(), datjit_core::model::VolumeSpec::Exact(10));

        let engine = GenerationEngine::new();
        let dataset = engine.generate(&doc).unwrap();

        assert_eq!(dataset.entities["User"].row_count(), 5);
        assert_eq!(dataset.entities["Order"].row_count(), 10);

        // Check all orders reference valid users
        let user_ids: std::collections::HashSet<String> = dataset.entities["User"]
            .rows
            .iter()
            .map(|r| r["id"].to_output_string())
            .collect();

        for row in &dataset.entities["Order"].rows {
            let user_ref = &row["user"];
            match user_ref {
                Value::Ref(entity, pk) => {
                    assert_eq!(entity, "User");
                    assert!(user_ids.contains(&pk.to_output_string()));
                }
                _ => panic!("expected Ref for user field"),
            }
        }
    }

    #[test]
    fn test_named_enum_resolved_to_variants() {
        let mut doc = DdlDocument::new("test");
        doc.seed = Some(42);

        // Define named enums
        doc.enums.insert(
            "Priority".into(),
            datjit_core::model::enum_def::EnumDef::simple(
                "Priority",
                vec!["critical", "high", "medium", "low"],
            ),
        );
        doc.enums.insert(
            "TaskStatus".into(),
            datjit_core::model::enum_def::EnumDef::simple(
                "TaskStatus",
                vec![
                    "backlog",
                    "todo",
                    "in_progress",
                    "review",
                    "done",
                    "cancelled",
                ],
            ),
        );

        let mut task = Entity::new("Task");
        task.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        // Named enum references (parsed as TypeExpr::Named by the parser)
        task.fields.insert(
            "priority".into(),
            Field::new("priority", TypeExpr::Named("Priority".into())),
        );
        task.fields.insert(
            "status".into(),
            Field::new("status", TypeExpr::Named("TaskStatus".into())),
        );

        doc.entities.insert("Task".into(), task);
        doc.volume
            .insert("Task".into(), datjit_core::model::VolumeSpec::Exact(50));

        let engine = GenerationEngine::new();
        let dataset = engine.generate(&doc).unwrap();

        let valid_priorities: std::collections::HashSet<&str> =
            ["critical", "high", "medium", "low"]
                .iter()
                .copied()
                .collect();
        let valid_statuses: std::collections::HashSet<&str> = [
            "backlog",
            "todo",
            "in_progress",
            "review",
            "done",
            "cancelled",
        ]
        .iter()
        .copied()
        .collect();

        for row in &dataset.entities["Task"].rows {
            let priority = row["priority"].to_output_string();
            let status = row["status"].to_output_string();
            assert!(
                valid_priorities.contains(priority.as_str()),
                "priority '{}' is not a valid enum variant",
                priority
            );
            assert!(
                valid_statuses.contains(status.as_str()),
                "status '{}' is not a valid enum variant",
                status
            );
        }
    }

    #[test]
    fn test_from_decorator_derives_email_without_coherence_group() {
        let mut doc = DdlDocument::new("test");
        doc.seed = Some(42);

        let mut user = Entity::new("User");
        user.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        user.fields.insert(
            "name".into(),
            Field::new(
                "name",
                TypeExpr::Semantic(SemanticType::new("person", "full")),
            ),
        );
        user.fields.insert(
            "email".into(),
            Field::new("email", TypeExpr::Semantic(SemanticType::new("email", "")))
                .with_decorators(vec![
                    Decorator::Unique,
                    Decorator::From(vec!["name".into()]),
                ]),
        );
        // NO coherence groups
        doc.entities.insert("User".into(), user);
        doc.volume
            .insert("User".into(), datjit_core::model::VolumeSpec::Exact(5));

        let engine = GenerationEngine::new();
        let dataset = engine.generate(&doc).unwrap();

        for row in &dataset.entities["User"].rows {
            let name = row["name"].to_output_string();
            let email = row["email"].to_output_string();
            let parts: Vec<&str> = name.split_whitespace().collect();
            if parts.len() >= 2 {
                let prefix = format!("{}.{}", parts[0].to_lowercase(), parts[1].to_lowercase());
                assert!(
                    email.starts_with(&prefix),
                    "email '{}' not derived from name '{}'",
                    email,
                    name
                );
            }
        }
    }
}
