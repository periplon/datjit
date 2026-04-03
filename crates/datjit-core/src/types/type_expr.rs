use serde::{Deserialize, Serialize};

use super::compound::CompoundType;
use super::primitive::PrimitiveType;
use super::reference::ReferenceType;
use super::semantic::SemanticType;

/// Unified type expression — every field's type resolves to this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    /// A primitive type: string, int, float, bool, datetime, etc.
    Primitive(PrimitiveType),
    /// A semantic type: person.full, email, address.city, etc.
    Semantic(SemanticType),
    /// An enum reference (inline or named)
    Enum(EnumRef),
    /// A reference to another entity
    Reference(ReferenceType),
    /// A compound type (list, map, tuple, nullable, union)
    Compound(CompoundType),
    /// A reference to a named reusable type from the `types:` section
    Named(String),
}

/// Reference to an enum, either inline or by name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumRef {
    /// `enum(active, inactive, suspended)`
    Inline(Vec<String>),
    /// Reference to a named enum: `Status`
    Named(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_expr_variants() {
        let prim = TypeExpr::Primitive(PrimitiveType::String(None));
        assert!(matches!(prim, TypeExpr::Primitive(_)));

        let sem = TypeExpr::Semantic(SemanticType::new("person", "full"));
        assert!(matches!(sem, TypeExpr::Semantic(_)));

        let enum_inline = TypeExpr::Enum(EnumRef::Inline(vec!["a".into(), "b".into(), "c".into()]));
        assert!(matches!(enum_inline, TypeExpr::Enum(EnumRef::Inline(_))));

        let enum_named = TypeExpr::Enum(EnumRef::Named("Status".into()));
        assert!(matches!(enum_named, TypeExpr::Enum(EnumRef::Named(_))));

        let named = TypeExpr::Named("Address".into());
        assert!(matches!(named, TypeExpr::Named(_)));
    }
}
