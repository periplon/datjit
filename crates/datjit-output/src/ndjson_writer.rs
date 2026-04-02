use std::io::Write;

use datjit_core::error::DatjitError;
use datjit_core::ports::generator::GeneratedDataSet;
use datjit_core::ports::OutputWriter;
use datjit_core::value::Value;

pub struct NdJsonWriter;

impl NdJsonWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NdJsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputWriter for NdJsonWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError> {
        for (_entity_name, entity_data) in &data.entities {
            write_entity_ndjson(entity_data, dest)?;
        }
        Ok(())
    }

    fn write_entity(
        &self,
        entity_name: &str,
        data: &GeneratedDataSet,
        dest: &mut dyn Write,
    ) -> Result<(), DatjitError> {
        let entity_data = data.entities.get(entity_name).ok_or_else(|| {
            DatjitError::Output(format!("entity not found: {entity_name}"))
        })?;
        write_entity_ndjson(entity_data, dest)
    }
}

fn write_entity_ndjson(
    entity_data: &datjit_core::ports::generator::EntityData,
    dest: &mut dyn Write,
) -> Result<(), DatjitError> {
    for row in &entity_data.rows {
        let obj: serde_json::Map<String, serde_json::Value> = entity_data
            .columns
            .iter()
            .filter_map(|col| {
                row.get(col).map(|v| (col.clone(), value_to_json(v)))
            })
            .collect();

        let line = serde_json::to_string(&serde_json::Value::Object(obj))
            .map_err(|e| DatjitError::Output(e.to_string()))?;
        dest.write_all(line.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))?;
        dest.write_all(b"\n")
            .map_err(|e| DatjitError::Output(e.to_string()))?;
    }
    Ok(())
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::Value::Number((*n).into()),
        Value::Float(n) => {
            serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::DateTime(s)
        | Value::Date(s)
        | Value::Time(s)
        | Value::Duration(s)
        | Value::Uuid(s) => serde_json::Value::String(s.clone()),
        Value::Bytes(b) => {
            let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            serde_json::Value::String(hex)
        }
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        Value::Map(pairs) => {
            let obj: serde_json::Map<String, serde_json::Value> = pairs
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Tuple(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        Value::Ref(_entity, pk) => value_to_json(pk),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::ports::generator::EntityData;
    use indexmap::IndexMap;

    fn sample_dataset() -> GeneratedDataSet {
        let mut dataset = GeneratedDataSet::new();
        let mut entity = EntityData::new("User", vec!["id".into(), "name".into()]);

        let mut row1 = IndexMap::new();
        row1.insert("id".into(), Value::Int(1));
        row1.insert("name".into(), Value::String("Alice".into()));
        entity.rows.push(row1);

        let mut row2 = IndexMap::new();
        row2.insert("id".into(), Value::Int(2));
        row2.insert("name".into(), Value::String("Bob".into()));
        entity.rows.push(row2);

        dataset.entities.insert("User".into(), entity);
        dataset
    }

    #[test]
    fn test_ndjson_one_line_per_row() {
        let dataset = sample_dataset();
        let writer = NdJsonWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);

        // Each line is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["name"], "Alice");
    }

    #[test]
    fn test_ndjson_no_pretty_printing() {
        let dataset = sample_dataset();
        let writer = NdJsonWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // Each line should be a single-line JSON (no indentation)
        for line in output.trim().lines() {
            assert!(!line.starts_with(' '));
            assert!(!line.contains('\n'));
        }
    }

    #[test]
    fn test_ndjson_write_entity() {
        let dataset = sample_dataset();
        let writer = NdJsonWriter::new();
        let mut buf = Vec::new();
        writer.write_entity("User", &dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
