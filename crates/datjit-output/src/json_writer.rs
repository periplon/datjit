use std::io::Write;

use datjit_core::error::DatjitError;
use datjit_core::ports::generator::GeneratedDataSet;
use datjit_core::ports::OutputWriter;
use datjit_core::value::Value;

pub struct JsonWriter {
    pub pretty: bool,
}

impl JsonWriter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl Default for JsonWriter {
    fn default() -> Self {
        Self { pretty: true }
    }
}

impl OutputWriter for JsonWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError> {
        let mut json_obj = serde_json::Map::new();

        for (entity_name, entity_data) in &data.entities {
            let rows: Vec<serde_json::Value> = entity_data
                .rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = entity_data
                        .columns
                        .iter()
                        .filter_map(|col| row.get(col).map(|v| (col.clone(), value_to_json(v))))
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();

            json_obj.insert(entity_name.clone(), serde_json::Value::Array(rows));
        }

        let json = serde_json::Value::Object(json_obj);
        let output = if self.pretty {
            serde_json::to_string_pretty(&json)
        } else {
            serde_json::to_string(&json)
        }
        .map_err(|e| DatjitError::Output(e.to_string()))?;

        dest.write_all(output.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))?;
        dest.write_all(b"\n")
            .map_err(|e| DatjitError::Output(e.to_string()))?;

        Ok(())
    }

    fn write_entity(
        &self,
        entity_name: &str,
        data: &GeneratedDataSet,
        dest: &mut dyn Write,
    ) -> Result<(), DatjitError> {
        let entity_data = data
            .entities
            .get(entity_name)
            .ok_or_else(|| DatjitError::Output(format!("entity not found: {entity_name}")))?;

        let rows: Vec<serde_json::Value> = entity_data
            .rows
            .iter()
            .map(|row| {
                let obj: serde_json::Map<String, serde_json::Value> = entity_data
                    .columns
                    .iter()
                    .filter_map(|col| row.get(col).map(|v| (col.clone(), value_to_json(v))))
                    .collect();
                serde_json::Value::Object(obj)
            })
            .collect();

        let json = serde_json::Value::Array(rows);
        let output = if self.pretty {
            serde_json::to_string_pretty(&json)
        } else {
            serde_json::to_string(&json)
        }
        .map_err(|e| DatjitError::Output(e.to_string()))?;

        dest.write_all(output.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))?;
        dest.write_all(b"\n")
            .map_err(|e| DatjitError::Output(e.to_string()))?;

        Ok(())
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::Value::Number((*n).into()),
        Value::Float(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::DateTime(s)
        | Value::Date(s)
        | Value::Time(s)
        | Value::Duration(s)
        | Value::Uuid(s) => serde_json::Value::String(s.clone()),
        Value::Bytes(b) => {
            // Simple hex encoding for bytes
            let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            serde_json::Value::String(hex)
        }
        Value::List(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
        Value::Map(pairs) => {
            let obj: serde_json::Map<String, serde_json::Value> = pairs
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Tuple(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
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

        let mut entity = EntityData::new("User", vec!["id".into(), "name".into(), "active".into()]);
        let mut row = IndexMap::new();
        row.insert("id".into(), Value::Uuid("abc-123".into()));
        row.insert("name".into(), Value::String("Alice".into()));
        row.insert("active".into(), Value::Bool(true));
        entity.rows.push(row);

        let mut row2 = IndexMap::new();
        row2.insert("id".into(), Value::Uuid("def-456".into()));
        row2.insert("name".into(), Value::String("Bob".into()));
        row2.insert("active".into(), Value::Bool(false));
        entity.rows.push(row2);

        dataset.entities.insert("User".into(), entity);
        dataset
    }

    #[test]
    fn test_json_output() {
        let dataset = sample_dataset();
        let writer = JsonWriter::new(false);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["User"].is_array());
        assert_eq!(parsed["User"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["User"][0]["name"], "Alice");
        assert_eq!(parsed["User"][1]["active"], false);
    }

    #[test]
    fn test_json_pretty() {
        let dataset = sample_dataset();
        let writer = JsonWriter::new(true);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains('\n'));
        assert!(output.contains("  ")); // indentation
    }

    #[test]
    fn test_write_entity() {
        let dataset = sample_dataset();
        let writer = JsonWriter::new(false);
        let mut buf = Vec::new();
        writer.write_entity("User", &dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_value_to_json_ref() {
        let val = Value::Ref("User".into(), Box::new(Value::Uuid("abc".into())));
        let json = value_to_json(&val);
        assert_eq!(json, serde_json::Value::String("abc".into()));
    }

    #[test]
    fn test_value_to_json_null() {
        assert_eq!(value_to_json(&Value::Null), serde_json::Value::Null);
    }
}
