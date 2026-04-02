use serde::{Deserialize, Serialize};
use std::fmt;

/// Runtime value produced by generators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    DateTime(String), // ISO 8601 string representation
    Date(String),     // ISO 8601 date string
    Time(String),     // ISO 8601 time string
    Duration(String), // ISO 8601 duration string
    Uuid(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
    Tuple(Vec<Value>),
    /// A reference to another entity: (entity_name, pk_value)
    Ref(String, Box<Value>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::DateTime(s) => Some(s),
            Value::Date(s) => Some(s),
            Value::Time(s) => Some(s),
            Value::Duration(s) => Some(s),
            Value::Uuid(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*n),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Convert to a display string for output formats.
    pub fn to_output_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => format!("{n:.2}"),
            Value::String(s) => s.clone(),
            Value::DateTime(s)
            | Value::Date(s)
            | Value::Time(s)
            | Value::Duration(s)
            | Value::Uuid(s) => s.clone(),
            Value::Bytes(b) => {
                use serde::Serialize;
                let mut buf = Vec::new();
                let mut ser = serde_json::Serializer::new(&mut buf);
                b.serialize(&mut ser).unwrap();
                String::from_utf8(buf).unwrap()
            }
            Value::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_output_string()).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Map(pairs) => {
                let inner: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_output_string()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
            Value::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_output_string()).collect();
                format!("({})", inner.join(", "))
            }
            Value::Ref(_, pk) => pk.to_output_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_output_string())
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Bool(b) => b.hash(state),
            Value::Int(n) => n.hash(state),
            Value::Float(n) => n.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::DateTime(s)
            | Value::Date(s)
            | Value::Time(s)
            | Value::Duration(s)
            | Value::Uuid(s) => s.hash(state),
            Value::Bytes(b) => b.hash(state),
            Value::List(items) => items.hash(state),
            Value::Map(pairs) => {
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Tuple(items) => items.hash(state),
            Value::Ref(entity, pk) => {
                entity.hash(state);
                pk.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        let v = Value::Null;
        assert!(v.is_null());
        assert_eq!(v.to_output_string(), "");
    }

    #[test]
    fn test_string_accessors() {
        let v = Value::String("hello".into());
        assert_eq!(v.as_str(), Some("hello"));
        assert!(v.as_i64().is_none());
    }

    #[test]
    fn test_numeric_accessors() {
        let v = Value::Int(42);
        assert_eq!(v.as_i64(), Some(42));
        assert_eq!(v.as_f64(), Some(42.0));

        let v = Value::Float(3.14);
        assert_eq!(v.as_f64(), Some(3.14));
        assert!(v.as_i64().is_none());
    }

    #[test]
    fn test_ref_display() {
        let v = Value::Ref("User".into(), Box::new(Value::Uuid("abc-123".into())));
        assert_eq!(v.to_output_string(), "abc-123");
    }

    #[test]
    fn test_display_trait() {
        let v = Value::Int(42);
        assert_eq!(format!("{v}"), "42");
    }

    #[test]
    fn test_hash_equality() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Value::Int(42));
        set.insert(Value::Int(42));
        assert_eq!(set.len(), 1);
    }
}
