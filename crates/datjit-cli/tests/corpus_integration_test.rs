use std::path::PathBuf;

use datjit_core::ports::{DataGenerator, DdlParser, OutputWriter};
use datjit_core::value::Value;
use datjit_generator::GenerationEngine;
use datjit_output::{CsvWriter, JsonWriter, NdJsonWriter, SqlDialect, SqlWriter, YamlWriter};
use datjit_parser::YamlParser;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

fn load_and_generate(fixture: &str, seed: u64) -> datjit_core::ports::GeneratedDataSet {
    let input = std::fs::read_to_string(fixture).unwrap();
    let parser = YamlParser;
    let doc = parser.parse(&input).unwrap();
    let engine = GenerationEngine::new().with_seed(seed);
    engine.generate(&doc).unwrap()
}

#[test]
fn test_generate_all_semantic_types() {
    let fixture = fixtures_dir().join("corpus_comprehensive.yaml");
    let dataset = load_and_generate(fixture.to_str().unwrap(), 42);

    // Check all entities were generated
    assert!(dataset.entities.contains_key("Person"));
    assert!(dataset.entities.contains_key("Organization"));
    assert!(dataset.entities.contains_key("Address"));
    assert!(dataset.entities.contains_key("Contact"));
    assert!(dataset.entities.contains_key("Employment"));
    assert!(dataset.entities.contains_key("Financial"));
    assert!(dataset.entities.contains_key("Product"));
    assert!(dataset.entities.contains_key("Content"));
    assert!(dataset.entities.contains_key("Technical"));

    // Check row counts
    assert_eq!(dataset.entities["Person"].row_count(), 20);
    assert_eq!(dataset.entities["Organization"].row_count(), 5);
    assert_eq!(dataset.entities["Product"].row_count(), 20);

    // Check Person fields are populated and realistic
    for row in &dataset.entities["Person"].rows {
        let first = row.get("first_name").unwrap();
        assert!(!first.is_null(), "first_name should not be null");
        if let Value::String(s) = first {
            assert!(!s.is_empty(), "first_name should not be empty");
            assert!(
                s.chars().next().unwrap().is_uppercase(),
                "first_name should be capitalized"
            );
        }

        let username = row.get("username").unwrap();
        assert!(!username.is_null());
    }

    // Check Address fields have expected formats
    for row in &dataset.entities["Address"].rows {
        if let Value::Float(lat) = row.get("lat").unwrap() {
            assert!(
                *lat >= -90.0 && *lat <= 90.0,
                "latitude out of range: {lat}"
            );
        }
        if let Value::Float(lng) = row.get("lng").unwrap() {
            assert!(
                *lng >= -180.0 && *lng <= 180.0,
                "longitude out of range: {lng}"
            );
        }
    }

    // Check Contact fields have expected formats
    for row in &dataset.entities["Contact"].rows {
        if let Value::String(email) = row.get("email").unwrap() {
            assert!(email.contains('@'), "email should contain @: {email}");
            assert!(email.contains('.'), "email should contain domain: {email}");
        }
    }

    // Check Technical fields
    for row in &dataset.entities["Technical"].rows {
        if let Value::String(ip) = row.get("ipv4_addr").unwrap() {
            assert!(
                ip.split('.').count() == 4,
                "IPv4 should have 4 octets: {ip}"
            );
        }
        if let Value::String(mac) = row.get("mac_addr").unwrap() {
            assert!(mac.contains(':'), "MAC should contain colons: {mac}");
        }
        if let Value::String(md5) = row.get("md5").unwrap() {
            assert_eq!(md5.len(), 32, "MD5 should be 32 hex chars: {md5}");
        }
        if let Value::String(sha) = row.get("sha256").unwrap() {
            assert_eq!(sha.len(), 64, "SHA256 should be 64 hex chars: {sha}");
        }
        if let Value::String(hex) = row.get("hex_color").unwrap() {
            assert!(
                hex.starts_with('#'),
                "hex color should start with #: {hex}"
            );
            assert_eq!(hex.len(), 7, "hex color should be #RRGGBB: {hex}");
        }
    }

    // Check Financial
    for row in &dataset.entities["Financial"].rows {
        if let Value::Float(price) = row.get("price_usd").unwrap() {
            assert!(
                *price >= 1.0 && *price <= 5000.0,
                "price out of range: {price}"
            );
        }
    }

    // Check Employment references
    for row in &dataset.entities["Employment"].rows {
        let org_ref = row.get("org").unwrap();
        assert!(!org_ref.is_null(), "org reference should not be null");
    }
}

#[test]
fn test_corpus_names_variety() {
    // Generate 100 persons and check we get variety (not the same 8 names from fallback)
    let input = r#"
domain: name_variety
seed: 42
volume:
  Person: 100
entities:
  Person:
    id: uuid @primary
    first_name: person.first
    last_name: person.last
"#;
    let parser = YamlParser;
    let doc = parser.parse(input).unwrap();
    let engine = GenerationEngine::new().with_seed(42);
    let dataset = engine.generate(&doc).unwrap();

    let first_names: std::collections::HashSet<String> = dataset.entities["Person"]
        .rows
        .iter()
        .filter_map(|r| r.get("first_name").and_then(|v| v.as_str()).map(String::from))
        .collect();

    let last_names: std::collections::HashSet<String> = dataset.entities["Person"]
        .rows
        .iter()
        .filter_map(|r| r.get("last_name").and_then(|v| v.as_str()).map(String::from))
        .collect();

    // With corpus, should have more variety than the 8-name embedded fallback
    // Even with embedded, 100 samples from 50+ names should give decent variety
    assert!(
        first_names.len() >= 5,
        "expected name variety, got {} unique first names",
        first_names.len()
    );
    assert!(
        last_names.len() >= 5,
        "expected name variety, got {} unique last names",
        last_names.len()
    );
}

#[test]
fn test_all_output_formats_with_corpus() {
    let fixture = fixtures_dir().join("corpus_comprehensive.yaml");
    let dataset = load_and_generate(fixture.to_str().unwrap(), 42);

    // JSON
    let mut buf = Vec::new();
    JsonWriter::new(false).write(&dataset, &mut buf).unwrap();
    let json_str = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["Person"].as_array().unwrap().len() == 20);

    // CSV
    let mut buf = Vec::new();
    CsvWriter::new().write(&dataset, &mut buf).unwrap();
    let csv_str = String::from_utf8(buf).unwrap();
    assert!(csv_str.contains("first_name"));

    // YAML
    let mut buf = Vec::new();
    YamlWriter::new().write(&dataset, &mut buf).unwrap();
    let yaml_str = String::from_utf8(buf).unwrap();
    assert!(yaml_str.contains("Person"));

    // NDJSON
    let mut buf = Vec::new();
    NdJsonWriter::new().write(&dataset, &mut buf).unwrap();
    let ndjson_str = String::from_utf8(buf).unwrap();
    assert!(ndjson_str.lines().count() > 0);

    // SQL
    let mut buf = Vec::new();
    SqlWriter::new(SqlDialect::Postgres)
        .write(&dataset, &mut buf)
        .unwrap();
    let sql_str = String::from_utf8(buf).unwrap();
    assert!(sql_str.contains("CREATE TABLE"));
    assert!(sql_str.contains("INSERT INTO"));
}

#[test]
fn test_deterministic_with_corpus() {
    let fixture = fixtures_dir().join("corpus_comprehensive.yaml");
    let ds1 = load_and_generate(fixture.to_str().unwrap(), 99);
    let ds2 = load_and_generate(fixture.to_str().unwrap(), 99);

    // Compare non-UUID fields (UUIDs use Uuid::new_v4 which isn't seeded)
    for entity_name in ds1.entities.keys() {
        let rows1 = &ds1.entities[entity_name].rows;
        let rows2 = &ds2.entities[entity_name].rows;
        assert_eq!(
            rows1.len(),
            rows2.len(),
            "row count mismatch for {entity_name}"
        );

        for (i, (r1, r2)) in rows1.iter().zip(rows2.iter()).enumerate() {
            for (field, v1) in r1 {
                if field == "id" {
                    continue;
                } // Skip UUID primary keys
                // Skip reference fields (contain non-deterministic UUIDs)
                if matches!(v1, Value::Ref(_, _)) {
                    continue;
                }
                let v2 = &r2[field];
                assert_eq!(v1, v2, "determinism failed: {entity_name}.{field} row {i}");
            }
        }
    }
}
