use crate::error::DatjitError;
use crate::types::SemanticType;
use crate::value::Value;

/// Port for providing corpus data for semantic type generation.
pub trait CorpusProvider {
    /// Sample a single value for a semantic type.
    fn sample(&self, semantic: &SemanticType, rng: &mut dyn RngCore) -> Result<Value, DatjitError>;

    /// Get available locales.
    fn available_locales(&self) -> Vec<String>;

    /// Set the active locale.
    fn set_locale(&mut self, locale: &str) -> Result<(), DatjitError>;
}

/// Re-export rand::RngCore so adapters don't need to depend on rand directly.
pub use rand::RngCore;
