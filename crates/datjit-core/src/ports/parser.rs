use crate::error::DatjitError;
use crate::model::DdlDocument;

/// Port for parsing DDL schema input into a domain model.
pub trait DdlParser {
    fn parse(&self, input: &str) -> Result<DdlDocument, DatjitError>;
}
