use indexmap::IndexMap;

use datjit_core::error::DatjitError;
use datjit_core::model::decorator::Decorator;
use datjit_core::model::DdlDocument;
use datjit_core::ports::generator::{DataGenerator, EntityData, GeneratedDataSet};
use datjit_core::value::Value;

use crate::coherence::generate_coherence_groups;
use crate::constraint::enforce_rules;
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

            // Collect the set of fields in coherence groups
            let coherence_field_set: std::collections::HashSet<String> = entity
                .coherence_groups
                .values()
                .flatten()
                .cloned()
                .collect();

            let mut entity_data = EntityData::new(entity_name.clone(), columns);

            for _ in 0..volume {
                let mut row = IndexMap::new();
                let mut rule_attempts = 0;
                const MAX_RULE_RETRIES: usize = 10;

                loop {
                    row.clear();

                    // Step 1: Generate coherence groups first
                    let coherence_values =
                        generate_coherence_groups(entity, &mut ctx)?;
                    for (k, v) in &coherence_values {
                        row.insert(k.clone(), v.clone());
                    }

                    // Step 2: Generate non-derived, non-coherence fields
                    for (field_name, field) in &entity.fields {
                        // Skip fields already populated by coherence groups
                        if coherence_field_set.contains(field_name)
                            && row.contains_key(field_name)
                        {
                            continue;
                        }

                        // Skip derived fields (evaluated later)
                        if field.is_derived() {
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

                    // Step 3: Evaluate @derived fields
                    for (field_name, field) in &entity.fields {
                        if !field.is_derived() {
                            continue;
                        }
                        for dec in &field.decorators {
                            if let Decorator::Derived(expr) = dec {
                                let derived_val =
                                    evaluate_derived(expr, &row, &ctx.generated)?;
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
                    let rule_result = enforce_rules(
                        &doc.rules,
                        entity_name,
                        &row,
                        &ctx.generated,
                        &mut ctx.rng,
                    );

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

            dataset.entities.insert(entity_name.clone(), entity_data);
        }

        Ok(dataset)
    }
}

fn generate_primary_key(_entity_name: &str, _ctx: &mut GenerationContext) -> Value {
    Value::Uuid(uuid::Uuid::new_v4().to_string())
}

fn generate_auto_field(field_name: &str, _ctx: &mut GenerationContext) -> Value {
    match field_name {
        "created_at" | "updated_at" => {
            Value::DateTime("2025-01-15T10:30:00".into())
        }
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
            Field::new("name", TypeExpr::Semantic(SemanticType::new("person", "full"))),
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

        let unique_count = emails.iter().collect::<std::collections::HashSet<_>>().len();
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
}
