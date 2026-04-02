pub mod csv_writer;
pub mod json_writer;
pub mod ndjson_writer;
pub mod sql_writer;
pub mod yaml_writer;

pub use csv_writer::CsvWriter;
pub use json_writer::JsonWriter;
pub use ndjson_writer::NdJsonWriter;
pub use sql_writer::{SqlDialect, SqlWriter};
pub use yaml_writer::YamlWriter;

/// Supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
    NdJson,
    Yaml,
    Sql,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "ndjson" | "jsonl" => Some(Self::NdJson),
            "yaml" | "yml" => Some(Self::Yaml),
            "sql" => Some(Self::Sql),
            _ => None,
        }
    }
}
