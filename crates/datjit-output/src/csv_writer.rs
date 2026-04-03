use std::io::Write;

use datjit_core::error::DatjitError;
use datjit_core::ports::generator::GeneratedDataSet;
use datjit_core::ports::OutputWriter;

pub struct CsvWriter;

impl CsvWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CsvWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputWriter for CsvWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError> {
        for (_entity_name, entity_data) in &data.entities {
            write_entity_csv(entity_data, dest)?;
        }
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
        write_entity_csv(entity_data, dest)
    }
}

fn write_entity_csv(
    entity_data: &datjit_core::ports::generator::EntityData,
    dest: &mut dyn Write,
) -> Result<(), DatjitError> {
    let mut wtr = csv::Writer::from_writer(dest);

    // Write header
    wtr.write_record(&entity_data.columns)
        .map_err(|e| DatjitError::Output(e.to_string()))?;

    // Write rows
    for row in &entity_data.rows {
        let record: Vec<String> = entity_data
            .columns
            .iter()
            .map(|col| {
                row.get(col)
                    .map(|v| {
                        if v.is_null() {
                            String::new()
                        } else {
                            v.to_output_string()
                        }
                    })
                    .unwrap_or_default()
            })
            .collect();
        wtr.write_record(&record)
            .map_err(|e| DatjitError::Output(e.to_string()))?;
    }

    wtr.flush()
        .map_err(|e| DatjitError::Output(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::ports::generator::EntityData;
    use datjit_core::value::Value;
    use indexmap::IndexMap;

    fn sample_dataset() -> GeneratedDataSet {
        let mut dataset = GeneratedDataSet::new();
        let mut entity = EntityData::new("User", vec!["id".into(), "name".into(), "age".into()]);

        let mut row1 = IndexMap::new();
        row1.insert("id".into(), Value::Int(1));
        row1.insert("name".into(), Value::String("Alice".into()));
        row1.insert("age".into(), Value::Int(30));
        entity.rows.push(row1);

        let mut row2 = IndexMap::new();
        row2.insert("id".into(), Value::Int(2));
        row2.insert("name".into(), Value::String("Bob".into()));
        row2.insert("age".into(), Value::Null);
        entity.rows.push(row2);

        dataset.entities.insert("User".into(), entity);
        dataset
    }

    #[test]
    fn test_csv_output_headers_and_rows() {
        let dataset = sample_dataset();
        let writer = CsvWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines[0], "id,name,age");
        assert_eq!(lines[1], "1,Alice,30");
    }

    #[test]
    fn test_csv_null_as_empty() {
        let dataset = sample_dataset();
        let writer = CsvWriter::new();
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        // Bob's age is null -> empty string
        assert_eq!(lines[2], "2,Bob,");
    }

    #[test]
    fn test_csv_write_entity() {
        let dataset = sample_dataset();
        let writer = CsvWriter::new();
        let mut buf = Vec::new();
        writer.write_entity("User", &dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("id,name,age"));
    }
}
