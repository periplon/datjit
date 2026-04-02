use std::fs;
use std::path::PathBuf;

use datjit_core::ports::{DataGenerator, DdlParser, OutputWriter};
use datjit_generator::GenerationEngine;
use datjit_output::JsonWriter;
use datjit_parser::YamlParser;

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
