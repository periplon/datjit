use thiserror::Error;

/// Central error type for the datjit system.
#[derive(Debug, Error)]
pub enum DatjitError {
    #[error("Parse error at {location}: {message}")]
    Parse { location: String, message: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown type: {0}")]
    UnknownType(String),

    #[error("Unknown entity reference: {0}")]
    UnknownEntity(String),

    #[error("Unknown enum reference: {0}")]
    UnknownEnum(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Constraint violation in {entity}: {message}")]
    ConstraintViolationAnnotated {
        entity: String,
        message: String,
        fields: Vec<String>,
    },

    #[error("Uniqueness exhausted for {entity}.{field} after {attempts} attempts")]
    UniquenessExhausted {
        entity: String,
        field: String,
        attempts: usize,
    },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Corpus error: {0}")]
    Corpus(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl DatjitError {
    pub fn parse(location: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse {
            location: location.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = DatjitError::parse("entities.User.name", "invalid type expression");
        assert_eq!(
            err.to_string(),
            "Parse error at entities.User.name: invalid type expression"
        );
    }

    #[test]
    fn test_validation_error() {
        let err = DatjitError::Validation("missing entity: Foo".into());
        assert_eq!(err.to_string(), "Validation error: missing entity: Foo");
    }
}
