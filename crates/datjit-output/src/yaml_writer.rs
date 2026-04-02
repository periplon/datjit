use std::io::Write;

use datjit_core::error::DatjitError;
use datjit_core::ports::generator::GeneratedDataSet;
use datjit_core::ports::OutputWriter;
use datjit_core::value::Value;

pub struct YamlWriter;

impl YamlWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for YamlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputWriter for YamlWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError> {
        let mut top = serde_yaml::Mapping::new();

        for (entity_name, entity_data) in &data.entities {
            let rows: Vec<serde_yaml::Value> = entity_data
                .rows
                .iter()
                .map(|row| {
                    let mut map = serde_yaml::Mapping::new();
                    for col in &entity_data.columns {
                        if let Some(v) = row.get(col) {
                            map.insert(
                                serde_yaml::Value::String(col.clone()),
                                value_to_yaml(v),
                            );
                        }
                    }
                    serde_yaml::Value::Mapping(map)
                })
                .collect();

            top.insert(
                serde_yaml::Value::String(entity_name.clone()),
                serde_yaml::Value::Sequence(rows),
            );
        }

        let yaml_str = serde_yaml::to_string(&serde_yaml::Value::Mapping(top))
            .map_err(|e| DatjitError::Output(e.to_string()))?;

        dest.write_all(yaml_str.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))?;

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

        let rows: Vec<serde_yaml::Value> = entity_data
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_yaml::Mapping::new();
                for col in &entity_data.columns {
                    if let Some(v) = row.get(col) {
                        map.insert(
                            serde_yaml::Value::String(col.clone()),
                            value_to_yaml(v),
                        );
                    }
                }
                serde_yaml::Value::Mapping(map)
            })
            .collect();

        let yaml_str = serde_yaml::to_string(&serde_yaml::Value::Sequence(rows))
            .map_err(|e| DatjitError::Output(e.to_string()))?;

        dest.write_all(yaml_str.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))?;

        Ok(())
    }
}

fn value_to_yaml(value: &Value) -> serde_yaml::Value {
    match value {
        Value::Null => serde_yaml::Value::Null,
        Value::Bool(b) => serde_yaml::Value::Bool(*b),
        Value::Int(n) => serde_yaml::Value::Number(serde_yaml::Number::from(*n)),
        Value::Float(n) => serde_yaml::Value::Number(serde_yaml::Number::from(*n)),
        Value::String(s) => serde_yaml::Value::String(s.clone()),
        Value::DateTime(s)
        | Value::Date(s)
        | Value::Time(s)
        | Value::Duration(s)
        | Value::Uuid(s) => serde_yaml::Value::String(s.clone()),
        Value::Bytes(b) => {
            let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            serde_yaml::Value::String(hex)
        }
        Value::List(items) => {
            serde_yaml::Value::Sequence(items.iter().map(value_to_yaml).collect())
        }
        Value::Map(pairs) => {
            let mut map = serde_yaml::Mapping::new();
            for (k, v) in pairs {
                map.insert(serde_yaml::Value::String(k.clone()), value_to_yaml(v));
            }
            serde_yaml::Value::Mapping(map)
        }
        Value::Tuple(items) => {
            serde_yaml::Value::Sequence(items.iter().map(value_to_yaml).collect())
        }
        Value::Ref(_entity, pk) => value_to_yaml(pk),
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

        let mut row1 = IndexMap::new();
        row1.insert("id".into(), Value::Int(1));
        row1.insert("name".into(), Value::String("Alice".into()));
        row1.insert("active".into(), Value::Bool(true));
        entity.rows.push(row1);

        let mut row2 = IndexMap::new();
        row2.insert("id".into(), Value::Int(2));
        row2.insert("name".into(), Value::Null);
        row2.insert("active".into(), Value::Bool(false));
        entity.rows.push(row2);

        dataset.entities.insert("User".into(), entity);
        dataset
    }

    #[test]
    fn test_yaml_output_structure() {
        let dataset = sample_dataset();
        let writer = YamlWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("User:"));
        assert!(output.contains("name: Alice"));
        assert!(output.contains("active: true"));
    }

    #[test]
    fn test_yaml_null_handling() {
        let dataset = sample_dataset();
        let writer = YamlWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("name: null") || output.contains("name: ~"));
    }

    #[test]
    fn test_yaml_write_entity() {
        let dataset = sample_dataset();
        let writer = YamlWriter::new();
        let mut buf = Vec::new();
        writer.write_entity("User", &dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // Entity-level output is a sequence, not keyed by entity name
        assert!(output.contains("name: Alice"));
        assert!(!output.contains("User:"));
    }
}
