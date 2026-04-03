use std::io::Write;

use datjit_core::error::DatjitError;
use datjit_core::ports::generator::{EntityData, GeneratedDataSet};
use datjit_core::ports::OutputWriter;
use datjit_core::value::Value;

/// SQL dialect for output generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    Postgres,
    Mysql,
    Sqlite,
}

impl SqlDialect {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "postgres" | "pg" | "postgresql" => Some(Self::Postgres),
            "mysql" => Some(Self::Mysql),
            "sqlite" => Some(Self::Sqlite),
            _ => None,
        }
    }
}

pub struct SqlWriter {
    pub dialect: SqlDialect,
    pub batch_size: usize,
    pub create_tables: bool,
}

impl SqlWriter {
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect,
            batch_size: 100,
            create_tables: true,
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn with_create_tables(mut self, create: bool) -> Self {
        self.create_tables = create;
        self
    }
}

impl Default for SqlWriter {
    fn default() -> Self {
        Self::new(SqlDialect::Postgres)
    }
}

impl OutputWriter for SqlWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError> {
        for (entity_name, entity_data) in &data.entities {
            write_entity_sql(entity_name, entity_data, self, dest)?;
            dest.write_all(b"\n")
                .map_err(|e| DatjitError::Output(e.to_string()))?;
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
        write_entity_sql(entity_name, entity_data, self, dest)
    }
}

fn write_entity_sql(
    entity_name: &str,
    entity_data: &EntityData,
    writer: &SqlWriter,
    dest: &mut dyn Write,
) -> Result<(), DatjitError> {
    let table_name = quote_identifier(entity_name, writer.dialect);

    if writer.create_tables {
        write_create_table(&table_name, entity_data, writer, dest)?;
    }

    if entity_data.rows.is_empty() {
        return Ok(());
    }

    // Batch inserts
    for chunk in entity_data.rows.chunks(writer.batch_size) {
        write_insert_batch(&table_name, entity_data, chunk, writer, dest)?;
    }

    Ok(())
}

fn write_create_table(
    table_name: &str,
    entity_data: &EntityData,
    writer: &SqlWriter,
    dest: &mut dyn Write,
) -> Result<(), DatjitError> {
    let mut w = |s: &str| {
        dest.write_all(s.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))
    };

    w(&format!("CREATE TABLE {table_name} (\n"))?;

    for (i, col) in entity_data.columns.iter().enumerate() {
        let sql_type = infer_sql_type(col, entity_data, writer.dialect);
        let col_name = quote_identifier(col, writer.dialect);
        let separator = if i < entity_data.columns.len() - 1 {
            ","
        } else {
            ""
        };
        w(&format!("  {col_name} {sql_type}{separator}\n"))?;
    }

    w(");\n\n")?;
    Ok(())
}

fn write_insert_batch(
    table_name: &str,
    entity_data: &EntityData,
    rows: &[indexmap::IndexMap<String, Value>],
    writer: &SqlWriter,
    dest: &mut dyn Write,
) -> Result<(), DatjitError> {
    let mut w = |s: &str| {
        dest.write_all(s.as_bytes())
            .map_err(|e| DatjitError::Output(e.to_string()))
    };

    let col_names: Vec<String> = entity_data
        .columns
        .iter()
        .map(|c| quote_identifier(c, writer.dialect))
        .collect();
    let col_list = col_names.join(", ");

    w(&format!("INSERT INTO {table_name} ({col_list}) VALUES\n"))?;

    for (row_idx, row) in rows.iter().enumerate() {
        let values: Vec<String> = entity_data
            .columns
            .iter()
            .map(|col| {
                row.get(col)
                    .map(|v| value_to_sql(v))
                    .unwrap_or_else(|| "NULL".to_string())
            })
            .collect();

        let separator = if row_idx < rows.len() - 1 { "," } else { ";" };
        w(&format!("  ({}){separator}\n", values.join(", ")))?;
    }

    Ok(())
}

fn value_to_sql(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => format!("{n}"),
        Value::String(s) => escape_sql_string(s),
        Value::DateTime(s)
        | Value::Date(s)
        | Value::Time(s)
        | Value::Duration(s)
        | Value::Uuid(s) => escape_sql_string(s),
        Value::Bytes(b) => {
            let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            format!("'\\x{hex}'")
        }
        Value::List(items) => {
            let inner: Vec<String> = items.iter().map(value_to_sql).collect();
            escape_sql_string(&format!("[{}]", inner.join(", ")))
        }
        Value::Map(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", escape_sql_string(k), value_to_sql(v)))
                .collect();
            escape_sql_string(&format!("{{{}}}", inner.join(", ")))
        }
        Value::Tuple(items) => {
            let inner: Vec<String> = items.iter().map(value_to_sql).collect();
            escape_sql_string(&format!("({})", inner.join(", ")))
        }
        Value::Ref(_entity, pk) => value_to_sql(pk),
    }
}

fn escape_sql_string(s: &str) -> String {
    let escaped = s.replace('\'', "''");
    format!("'{escaped}'")
}

fn quote_identifier(name: &str, dialect: SqlDialect) -> String {
    match dialect {
        SqlDialect::Postgres => format!("\"{name}\""),
        SqlDialect::Mysql => format!("`{name}`"),
        SqlDialect::Sqlite => format!("\"{name}\""),
    }
}

fn infer_sql_type(col: &str, entity_data: &EntityData, dialect: SqlDialect) -> String {
    // Look at the first non-null value for this column to infer its type
    let first_value = entity_data.rows.iter().find_map(|row| {
        row.get(col)
            .and_then(|v| if v.is_null() { None } else { Some(v) })
    });

    match first_value {
        Some(value) => value_to_sql_type(value, dialect),
        None => "TEXT".to_string(), // default to TEXT if all values are null
    }
}

fn value_to_sql_type(value: &Value, dialect: SqlDialect) -> String {
    match value {
        Value::Null => "TEXT".to_string(),
        Value::Bool(_) => match dialect {
            SqlDialect::Mysql => "BOOLEAN".to_string(),
            _ => "BOOLEAN".to_string(),
        },
        Value::Int(_) => match dialect {
            SqlDialect::Postgres => "BIGINT".to_string(),
            SqlDialect::Mysql => "BIGINT".to_string(),
            SqlDialect::Sqlite => "INTEGER".to_string(),
        },
        Value::Float(_) => match dialect {
            SqlDialect::Postgres => "DOUBLE PRECISION".to_string(),
            SqlDialect::Mysql => "DOUBLE".to_string(),
            SqlDialect::Sqlite => "REAL".to_string(),
        },
        Value::String(_) => "TEXT".to_string(),
        Value::DateTime(_) => match dialect {
            SqlDialect::Postgres => "TIMESTAMP".to_string(),
            SqlDialect::Mysql => "DATETIME".to_string(),
            SqlDialect::Sqlite => "TEXT".to_string(),
        },
        Value::Date(_) => match dialect {
            SqlDialect::Sqlite => "TEXT".to_string(),
            _ => "DATE".to_string(),
        },
        Value::Time(_) => match dialect {
            SqlDialect::Sqlite => "TEXT".to_string(),
            _ => "TIME".to_string(),
        },
        Value::Duration(_) => "TEXT".to_string(),
        Value::Uuid(_) => match dialect {
            SqlDialect::Postgres => "UUID".to_string(),
            _ => "TEXT".to_string(),
        },
        Value::Bytes(_) => match dialect {
            SqlDialect::Postgres => "BYTEA".to_string(),
            SqlDialect::Mysql => "BLOB".to_string(),
            SqlDialect::Sqlite => "BLOB".to_string(),
        },
        Value::List(_) | Value::Map(_) | Value::Tuple(_) => match dialect {
            SqlDialect::Postgres => "JSONB".to_string(),
            _ => "TEXT".to_string(),
        },
        Value::Ref(_, pk) => value_to_sql_type(pk, dialect),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::ports::generator::EntityData;
    use indexmap::IndexMap;

    fn sample_dataset() -> GeneratedDataSet {
        let mut dataset = GeneratedDataSet::new();
        let mut entity = EntityData::new(
            "users",
            vec!["id".into(), "name".into(), "email".into(), "age".into()],
        );

        let mut row1 = IndexMap::new();
        row1.insert("id".into(), Value::Int(1));
        row1.insert("name".into(), Value::String("Alice".into()));
        row1.insert("email".into(), Value::String("alice@example.com".into()));
        row1.insert("age".into(), Value::Int(30));
        entity.rows.push(row1);

        let mut row2 = IndexMap::new();
        row2.insert("id".into(), Value::Int(2));
        row2.insert("name".into(), Value::String("Bob's Place".into()));
        row2.insert("email".into(), Value::String("bob@example.com".into()));
        row2.insert("age".into(), Value::Null);
        entity.rows.push(row2);

        dataset.entities.insert("users".into(), entity);
        dataset
    }

    #[test]
    fn test_sql_create_table_and_inserts() {
        let dataset = sample_dataset();
        let writer = SqlWriter::new(SqlDialect::Postgres);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("CREATE TABLE \"users\""));
        assert!(output.contains("\"id\" BIGINT"));
        assert!(output.contains("\"name\" TEXT"));
        assert!(output.contains("INSERT INTO \"users\""));
        assert!(output.contains("'Alice'"));
        assert!(output.contains("NULL"));
    }

    #[test]
    fn test_sql_escaping_single_quotes() {
        let dataset = sample_dataset();
        let writer = SqlWriter::new(SqlDialect::Postgres);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // "Bob's Place" should be escaped to "Bob''s Place"
        assert!(output.contains("'Bob''s Place'"));
    }

    #[test]
    fn test_sql_mysql_dialect() {
        let dataset = sample_dataset();
        let writer = SqlWriter::new(SqlDialect::Mysql);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // MySQL uses backtick quoting
        assert!(output.contains("CREATE TABLE `users`"));
        assert!(output.contains("`id` BIGINT"));
    }

    #[test]
    fn test_sql_batch_inserts() {
        let mut dataset = GeneratedDataSet::new();
        let mut entity = EntityData::new("items", vec!["id".into()]);
        for i in 0..5 {
            let mut row = IndexMap::new();
            row.insert("id".into(), Value::Int(i));
            entity.rows.push(row);
        }
        dataset.entities.insert("items".into(), entity);

        let writer = SqlWriter::new(SqlDialect::Sqlite).with_batch_size(2);
        let mut buf = Vec::new();
        writer.write(&dataset, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // With batch_size=2 and 5 rows, we should get 3 INSERT statements
        let insert_count = output.matches("INSERT INTO").count();
        assert_eq!(insert_count, 3);
    }
}
