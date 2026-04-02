use serde::{Deserialize, Serialize};

use super::type_expr::TypeExpr;

/// Compound types that compose other types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompoundType {
    /// `[T]` — list of T
    List(Box<TypeExpr>),
    /// `{K: V}` — map with key type K, value type V
    Map(Box<TypeExpr>, Box<TypeExpr>),
    /// `(T1, T2, ...)` — tuple
    Tuple(Vec<TypeExpr>),
    /// `T?` — nullable (equivalent to T | null)
    Nullable(Box<TypeExpr>),
    /// `T1 | T2` — union type
    Union(Vec<TypeExpr>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::primitive::PrimitiveType;

    #[test]
    fn test_list() {
        let list = CompoundType::List(Box::new(TypeExpr::Primitive(PrimitiveType::String(None))));
        match list {
            CompoundType::List(inner) => {
                assert_eq!(*inner, TypeExpr::Primitive(PrimitiveType::String(None)));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_nullable() {
        let nullable =
            CompoundType::Nullable(Box::new(TypeExpr::Primitive(PrimitiveType::Int(None))));
        match nullable {
            CompoundType::Nullable(inner) => {
                assert_eq!(*inner, TypeExpr::Primitive(PrimitiveType::Int(None)));
            }
            _ => panic!("expected Nullable"),
        }
    }

    #[test]
    fn test_union() {
        let union = CompoundType::Union(vec![
            TypeExpr::Primitive(PrimitiveType::String(None)),
            TypeExpr::Primitive(PrimitiveType::Int(None)),
        ]);
        match union {
            CompoundType::Union(types) => assert_eq!(types.len(), 2),
            _ => panic!("expected Union"),
        }
    }
}
