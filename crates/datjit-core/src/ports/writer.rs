use std::io::Write;

use crate::error::DatjitError;
use crate::ports::generator::GeneratedDataSet;

/// Port for writing generated data to an output format.
pub trait OutputWriter {
    fn write(&self, data: &GeneratedDataSet, dest: &mut dyn Write) -> Result<(), DatjitError>;

    /// Write data for a specific entity only.
    fn write_entity(
        &self,
        entity_name: &str,
        data: &GeneratedDataSet,
        dest: &mut dyn Write,
    ) -> Result<(), DatjitError>;
}
