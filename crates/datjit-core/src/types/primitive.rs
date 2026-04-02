use serde::{Deserialize, Serialize};

/// Primitive types in the DDL type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    /// UTF-8 text, optional max length
    String(Option<usize>),
    /// Signed integer, optional bit width (8, 16, 32, 64)
    Int(Option<u8>),
    /// IEEE 754 float, optional bit width (32, 64)
    Float(Option<u8>),
    /// Fixed-point decimal(precision, scale)
    Decimal(u8, u8),
    /// true / false
    Bool,
    /// ISO 8601 timestamp
    DateTime,
    /// ISO 8601 date only
    Date,
    /// ISO 8601 time only
    Time,
    /// ISO 8601 duration
    Duration,
    /// UUID v4
    Uuid,
    /// Base64-encoded binary, optional max byte length
    Bytes(Option<usize>),
    /// Explicit null
    Null,
    /// Untyped / opaque JSON
    Any,
}

impl PrimitiveType {
    /// Try to parse a primitive type name, optionally with parameters.
    /// Returns None if the name is not a known primitive.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "string" => Some(PrimitiveType::String(None)),
            "int" => Some(PrimitiveType::Int(None)),
            "float" => Some(PrimitiveType::Float(None)),
            "bool" => Some(PrimitiveType::Bool),
            "datetime" => Some(PrimitiveType::DateTime),
            "date" => Some(PrimitiveType::Date),
            "time" => Some(PrimitiveType::Time),
            "duration" => Some(PrimitiveType::Duration),
            "uuid" => Some(PrimitiveType::Uuid),
            "bytes" => Some(PrimitiveType::Bytes(None)),
            "null" => Some(PrimitiveType::Null),
            "any" => Some(PrimitiveType::Any),
            _ => None,
        }
    }

    /// Check if a name is a known primitive type.
    pub fn is_primitive_name(name: &str) -> bool {
        matches!(
            name,
            "string"
                | "int"
                | "float"
                | "bool"
                | "datetime"
                | "date"
                | "time"
                | "duration"
                | "uuid"
                | "bytes"
                | "null"
                | "any"
                | "decimal"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name() {
        assert_eq!(
            PrimitiveType::from_name("string"),
            Some(PrimitiveType::String(None))
        );
        assert_eq!(
            PrimitiveType::from_name("int"),
            Some(PrimitiveType::Int(None))
        );
        assert_eq!(PrimitiveType::from_name("bool"), Some(PrimitiveType::Bool));
        assert_eq!(PrimitiveType::from_name("uuid"), Some(PrimitiveType::Uuid));
        assert_eq!(PrimitiveType::from_name("unknown"), None);
    }

    #[test]
    fn test_is_primitive_name() {
        assert!(PrimitiveType::is_primitive_name("string"));
        assert!(PrimitiveType::is_primitive_name("decimal"));
        assert!(!PrimitiveType::is_primitive_name("person"));
        assert!(!PrimitiveType::is_primitive_name("email"));
    }

    #[test]
    fn test_parameterized() {
        let s = PrimitiveType::String(Some(100));
        assert_eq!(s, PrimitiveType::String(Some(100)));

        let d = PrimitiveType::Decimal(10, 2);
        assert_eq!(d, PrimitiveType::Decimal(10, 2));
    }
}
