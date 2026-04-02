use crate::error::DatjitError;
use crate::model::DdlDocument;
use crate::value::Value;
use indexmap::IndexMap;

/// The result of data generation: entities mapped to their generated rows.
#[derive(Debug, Clone)]
pub struct GeneratedDataSet {
    /// Entity name -> rows, where each row is field_name -> value.
    /// Outer IndexMap preserves entity order, inner IndexMap preserves field order.
    pub entities: IndexMap<String, EntityData>,
}

/// Generated data for a single entity.
#[derive(Debug, Clone)]
pub struct EntityData {
    pub name: String,
    /// Column names in definition order.
    pub columns: Vec<String>,
    /// Rows, each row is a map of field_name -> value.
    pub rows: Vec<IndexMap<String, Value>>,
}

impl EntityData {
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            rows: Vec::new(),
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

impl GeneratedDataSet {
    pub fn new() -> Self {
        Self {
            entities: IndexMap::new(),
        }
    }

    pub fn total_rows(&self) -> usize {
        self.entities.values().map(|e| e.row_count()).sum()
    }
}

impl Default for GeneratedDataSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Port for generating data from a parsed DDL document.
pub trait DataGenerator {
    fn generate(&self, doc: &DdlDocument) -> Result<GeneratedDataSet, DatjitError>;
}
