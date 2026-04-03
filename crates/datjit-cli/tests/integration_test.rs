use std::fs;
use std::path::PathBuf;

use datjit_core::ports::{DataGenerator, DdlParser, OutputWriter};
use datjit_core::value::Value;
use datjit_generator::GenerationEngine;
use datjit_output::JsonWriter;
use datjit_parser::YamlParser;

/// Helper: parse a fixture, generate data, and return the dataset.
/// Panics with a clear message on failure.
fn parse_and_generate(
    fixture: &str,
) -> datjit_core::ports::generator::GeneratedDataSet {
    let yaml = fs::read_to_string(fixtures_dir().join(fixture))
        .unwrap_or_else(|_| panic!("failed to read {fixture}"));
    let parser = YamlParser;
    let doc = parser
        .parse(&yaml)
        .unwrap_or_else(|e| panic!("failed to parse {fixture}: {e}"));
    let engine = GenerationEngine::new().with_seed(42);
    engine
        .generate(&doc)
        .unwrap_or_else(|e| panic!("failed to generate {fixture}: {e}"))
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_parse_minimal_yaml_and_generate_json() {
    let yaml = fs::read_to_string(fixtures_dir().join("minimal.yaml"))
        .expect("failed to read minimal.yaml");

    let parser = YamlParser;
    let doc = parser.parse(&yaml).expect("failed to parse minimal.yaml");

    assert_eq!(doc.domain, "test_minimal");
    assert_eq!(doc.entities.len(), 1);
    assert!(doc.entities.contains_key("User"));

    let engine = GenerationEngine::new().with_seed(42);
    let dataset = engine.generate(&doc).expect("failed to generate data");

    assert!(dataset.entities.contains_key("User"));
    let user_data = &dataset.entities["User"];
    assert_eq!(user_data.row_count(), 10); // volume: User: 10

    // Write as JSON and verify it is valid
    let writer = JsonWriter::new(false);
    let mut buf = Vec::new();
    writer
        .write(&dataset, &mut buf)
        .expect("failed to write JSON");

    let output = String::from_utf8(buf).expect("output is not valid UTF-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("output is not valid JSON");

    assert!(parsed["User"].is_array());
    assert_eq!(parsed["User"].as_array().unwrap().len(), 10);

    // Each user row should have the expected fields
    let first_user = &parsed["User"][0];
    assert!(first_user.get("id").is_some());
    assert!(first_user.get("name").is_some());
    assert!(first_user.get("email").is_some());
    assert!(first_user.get("age").is_some());
    assert!(first_user.get("active").is_some());
}

#[test]
fn test_parse_project_management_yaml_entity_counts() {
    let yaml = fs::read_to_string(fixtures_dir().join("project_management.yaml"))
        .expect("failed to read project_management.yaml");

    let parser = YamlParser;
    let doc = parser
        .parse(&yaml)
        .expect("failed to parse project_management.yaml");

    assert_eq!(doc.domain, "project_management");
    assert_eq!(doc.entities.len(), 4);
    assert!(doc.entities.contains_key("Organization"));
    assert!(doc.entities.contains_key("User"));
    assert!(doc.entities.contains_key("Project"));
    assert!(doc.entities.contains_key("Task"));

    // Verify enum definitions
    assert!(doc.enums.contains_key("Priority"));
    assert_eq!(doc.enums["Priority"].variants.len(), 4);
    assert!(doc.enums.contains_key("TaskStatus"));
    assert_eq!(doc.enums["TaskStatus"].variants.len(), 6);

    let engine = GenerationEngine::new().with_seed(42);
    let dataset = engine
        .generate(&doc)
        .expect("failed to generate project_management data");

    // Verify entity row counts match the volume spec
    assert_eq!(dataset.entities["Organization"].row_count(), 5);
    assert_eq!(dataset.entities["User"].row_count(), 50);
    assert_eq!(dataset.entities["Project"].row_count(), 20);
    assert_eq!(dataset.entities["Task"].row_count(), 200);
}

#[test]
fn test_deterministic_output_same_seed() {
    let yaml = fs::read_to_string(fixtures_dir().join("minimal.yaml"))
        .expect("failed to read minimal.yaml");

    let parser = YamlParser;
    let doc = parser.parse(&yaml).expect("failed to parse minimal.yaml");

    let writer = JsonWriter::new(false);

    // Helper: extract non-id fields from JSON output for comparison.
    // Primary key UUIDs use uuid::Uuid::new_v4() (system entropy), so they
    // are not deterministic. All seeded fields (name, email, age, active)
    // must be identical across runs with the same seed.
    let strip_ids = |json_str: &str| -> serde_json::Value {
        let mut val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        if let Some(entities) = val.as_object_mut() {
            for (_name, rows) in entities.iter_mut() {
                if let Some(arr) = rows.as_array_mut() {
                    for row in arr.iter_mut() {
                        if let Some(obj) = row.as_object_mut() {
                            obj.remove("id");
                        }
                    }
                }
            }
        }
        val
    };

    // Generate twice with the same seed
    let engine1 = GenerationEngine::new().with_seed(12345);
    let dataset1 = engine1.generate(&doc).expect("first generation failed");
    let mut buf1 = Vec::new();
    writer
        .write(&dataset1, &mut buf1)
        .expect("first write failed");
    let output1 = String::from_utf8(buf1).unwrap();

    let engine2 = GenerationEngine::new().with_seed(12345);
    let dataset2 = engine2.generate(&doc).expect("second generation failed");
    let mut buf2 = Vec::new();
    writer
        .write(&dataset2, &mut buf2)
        .expect("second write failed");
    let output2 = String::from_utf8(buf2).unwrap();

    let stripped1 = strip_ids(&output1);
    let stripped2 = strip_ids(&output2);
    assert_eq!(
        stripped1, stripped2,
        "same seed must produce identical seeded fields"
    );

    // Verify a different seed produces different seeded fields
    let engine3 = GenerationEngine::new().with_seed(99999);
    let dataset3 = engine3.generate(&doc).expect("third generation failed");
    let mut buf3 = Vec::new();
    writer
        .write(&dataset3, &mut buf3)
        .expect("third write failed");
    let output3 = String::from_utf8(buf3).unwrap();

    let stripped3 = strip_ids(&output3);
    assert_ne!(
        stripped1, stripped3,
        "different seeds should produce different output"
    );
}

#[test]
fn test_named_enums_generate_valid_variants() {
    let yaml = fs::read_to_string(fixtures_dir().join("project_management.yaml"))
        .expect("failed to read project_management.yaml");

    let parser = YamlParser;
    let doc = parser.parse(&yaml).expect("failed to parse");

    let engine = GenerationEngine::new().with_seed(42);
    let dataset = engine.generate(&doc).unwrap();
    let tasks = &dataset.entities["Task"];

    let valid_priorities: std::collections::HashSet<&str> =
        ["critical", "high", "medium", "low"].iter().copied().collect();
    let valid_statuses: std::collections::HashSet<&str> =
        ["backlog", "todo", "in_progress", "review", "done", "cancelled"]
            .iter()
            .copied()
            .collect();

    for row in &tasks.rows {
        let priority = row["priority"].to_output_string();
        let status = row["status"].to_output_string();
        assert!(
            valid_priorities.contains(priority.as_str()),
            "Task priority '{}' is not a valid Priority variant",
            priority
        );
        assert!(
            valid_statuses.contains(status.as_str()),
            "Task status '{}' is not a valid TaskStatus variant",
            status
        );
    }
}

#[test]
fn test_from_decorator_derives_email_from_name() {
    let yaml = fs::read_to_string(fixtures_dir().join("minimal.yaml"))
        .expect("failed to read minimal.yaml");

    let parser = YamlParser;
    let doc = parser.parse(&yaml).expect("failed to parse");

    let engine = GenerationEngine::new().with_seed(42);
    let dataset = engine.generate(&doc).unwrap();
    let users = &dataset.entities["User"];

    for row in &users.rows {
        let name = row["name"].to_output_string();
        let email = row["email"].to_output_string();
        let name_parts: Vec<&str> = name.split_whitespace().collect();
        if name_parts.len() >= 2 {
            let expected_local = format!(
                "{}.{}",
                name_parts[0].to_lowercase(),
                name_parts[1].to_lowercase()
            );
            assert!(
                email.starts_with(&expected_local),
                "Row email '{}' should derive from name '{}' (expected prefix '{}')",
                email,
                name,
                expected_local
            );
        }
        assert!(email.contains('@'));
    }
}

// ---------------------------------------------------------------------------
// Fixture coverage tests: ensure every DDL feature fixture parses & generates
// ---------------------------------------------------------------------------

#[test]
fn test_fixture_primitives_and_params() {
    let ds = parse_and_generate("primitives_and_params.yaml");
    let rows = &ds.entities["AllPrimitives"];
    assert_eq!(rows.row_count(), 20);
    // Verify all primitive fields are present
    let r = &rows.rows[0];
    for field in &[
        "id", "label", "label_bounded", "count", "count_32", "ratio", "ratio_32",
        "price", "active", "created", "birthday", "alarm", "elapsed", "token",
        "payload", "payload_small",
    ] {
        assert!(r.contains_key(*field), "missing field: {field}");
    }
}

#[test]
fn test_fixture_semantic_types() {
    let ds = parse_and_generate("semantic_types.yaml");
    assert_eq!(ds.entities.len(), 9);
    for (name, data) in &ds.entities {
        assert_eq!(data.row_count(), 10, "{name} should have 10 rows");
    }
    // Spot-check a few semantic values
    let person = &ds.entities["Person"].rows[0];
    let name = person["full_name"].to_output_string();
    assert!(name.contains(' '), "person.full should have first+last: {name}");

    let contact = &ds.entities["Contact"].rows[0];
    let email = contact["email"].to_output_string();
    assert!(email.contains('@'), "email should contain @: {email}");
}

#[test]
fn test_fixture_enums_and_distributions() {
    let ds = parse_and_generate("enums_and_distributions.yaml");
    let enums = &ds.entities["EnumSampler"];
    assert_eq!(enums.row_count(), 100);

    let valid_colors: std::collections::HashSet<&str> =
        ["red", "green", "blue", "yellow"].iter().copied().collect();
    let valid_tiers: std::collections::HashSet<&str> =
        ["free", "pro", "enterprise"].iter().copied().collect();
    let valid_priorities: std::collections::HashSet<&str> =
        ["critical", "high", "medium", "low"].iter().copied().collect();

    for row in &enums.rows {
        let color = row["color"].to_output_string();
        assert!(valid_colors.contains(color.as_str()), "invalid color: {color}");
        let tier = row["tier"].to_output_string();
        assert!(valid_tiers.contains(tier.as_str()), "invalid tier: {tier}");
        let priority = row["priority"].to_output_string();
        assert!(valid_priorities.contains(priority.as_str()), "invalid priority: {priority}");
    }

    // Verify distributions produce values in range
    let dists = &ds.entities["DistSampler"];
    for row in &dists.rows {
        if let Value::Int(score) = &row["score"] {
            assert!(*score >= 0 && *score <= 100, "score out of range: {score}");
        }
    }
}

#[test]
fn test_fixture_decorators() {
    let ds = parse_and_generate("decorators.yaml");
    let constrained = &ds.entities["Constrained"];
    assert_eq!(constrained.row_count(), 50);

    // Verify range constraints
    for row in &constrained.rows {
        if let Value::Int(age) = &row["age"] {
            assert!(*age >= 18 && *age <= 65, "age out of range: {age}");
        }
        // Verify length constraints
        let title = row["title"].to_output_string();
        assert!(title.len() >= 5, "title too short: '{title}'");
    }

    // Verify uniqueness on emails
    let emails: Vec<_> = constrained.rows.iter().map(|r| r["email"].to_output_string()).collect();
    let unique: std::collections::HashSet<_> = emails.iter().collect();
    assert_eq!(emails.len(), unique.len(), "emails should be unique");

    // Verify pattern templates
    let patterned = &ds.entities["Patterned"];
    for row in &patterned.rows {
        let sku = row["sku"].to_output_string();
        assert!(sku.starts_with("SKU-"), "sku should start with SKU-: {sku}");
    }
}

#[test]
fn test_fixture_references() {
    let ds = parse_and_generate("references.yaml");
    assert_eq!(ds.entities["Department"].row_count(), 5);
    assert_eq!(ds.entities["Employee"].row_count(), 30);
    assert_eq!(ds.entities["Project"].row_count(), 10);
    assert_eq!(ds.entities["Task"].row_count(), 50);

    // Verify references are present (either Ref or Null for optionals)
    for row in &ds.entities["Task"].rows {
        assert!(row.contains_key("project"));
        assert!(row.contains_key("assignee"));
        assert!(row.contains_key("parent"));
    }
}

#[test]
fn test_fixture_coherence_groups() {
    let ds = parse_and_generate("coherence_groups.yaml");

    // Verify location coherence: timezone matches city/state
    let offices = &ds.entities["Office"];
    for row in &offices.rows {
        let tz = row["timezone"].to_output_string();
        assert!(
            tz.contains('/'),
            "timezone should be IANA format: {tz}"
        );
        let phone = row["phone"].to_output_string();
        assert!(
            phone.starts_with("+1-"),
            "phone should start with +1-: {phone}"
        );
    }

    // Verify identity coherence: email derived from first+last
    let employees = &ds.entities["Employee"];
    for row in &employees.rows {
        let first = row["first_name"].to_output_string();
        let email = row["email"].to_output_string();
        assert!(
            email.starts_with(&first.to_lowercase()),
            "email '{email}' should start with first name '{first}'"
        );
    }
}

#[test]
fn test_fixture_entity_meta() {
    let ds = parse_and_generate("entity_meta.yaml");

    // Verify @timestamps adds created_at and updated_at
    let users = &ds.entities["User"];
    assert_eq!(users.row_count(), 10);
    for row in &users.rows {
        assert!(
            row.contains_key("created_at"),
            "User should have created_at from @timestamps"
        );
        assert!(
            row.contains_key("updated_at"),
            "User should have updated_at from @timestamps"
        );
    }
}

#[test]
fn test_fixture_rules() {
    let ds = parse_and_generate("rules.yaml");
    let users = &ds.entities["User"];
    assert_eq!(users.row_count(), 10);

    // @strict rule: age >= 18
    for row in &users.rows {
        if let Value::Int(age) = &row["age"] {
            assert!(*age >= 18, "strict rule violated: age {age} < 18");
        }
    }

    // @strict rule: amount > 0
    let orders = &ds.entities["Order"];
    for row in &orders.rows {
        if let Value::Float(amount) = &row["amount"] {
            assert!(*amount > 0.0, "strict rule violated: amount {amount} <= 0");
        }
    }
}

#[test]
fn test_fixture_derived_fields() {
    let ds = parse_and_generate("derived_fields.yaml");
    let products = &ds.entities["Product"];
    assert_eq!(products.row_count(), 20);

    for row in &products.rows {
        // Verify slug is derived from name
        assert!(row.contains_key("slug"), "Product should have derived slug");
        assert!(row.contains_key("name_lower"), "Product should have derived name_lower");
        assert!(row.contains_key("tax_amount"), "Product should have derived tax_amount");
    }
}

#[test]
fn test_fixture_compound_types() {
    let ds = parse_and_generate("compound_types.yaml");
    let records = &ds.entities["Record"];
    assert_eq!(records.row_count(), 15);

    for row in &records.rows {
        // List field should be a List
        assert!(
            matches!(&row["tags"], Value::List(_)),
            "tags should be a list, got: {:?}",
            row["tags"]
        );
        // Tuple should be a Tuple
        assert!(
            matches!(&row["coordinates"], Value::Tuple(_)),
            "coordinates should be a tuple, got: {:?}",
            row["coordinates"]
        );
    }
}

#[test]
fn test_fixture_named_types() {
    let ds = parse_and_generate("named_types.yaml");
    assert_eq!(ds.entities["Customer"].row_count(), 10);
    assert_eq!(ds.entities["Vendor"].row_count(), 5);

    // Named enum Tier should resolve to valid variants
    let valid_tiers: std::collections::HashSet<&str> =
        ["free", "basic", "premium", "enterprise"].iter().copied().collect();
    for row in &ds.entities["Customer"].rows {
        let tier = row["tier"].to_output_string();
        assert!(valid_tiers.contains(tier.as_str()), "invalid tier: {tier}");
    }
}
