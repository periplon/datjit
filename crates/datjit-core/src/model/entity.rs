use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::decorator::Decorator;
use super::trigger::Trigger;
use crate::types::TypeExpr;

/// An entity definition from the `entities:` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    /// Entity-level decorators from `_meta`
    pub meta: Vec<Decorator>,
    /// Fields in definition order
    pub fields: IndexMap<String, Field>,
    /// Coherence groups from `_coherence`
    pub coherence_groups: IndexMap<String, Vec<String>>,
    /// Triggers from `_triggers`
    pub triggers: Vec<Trigger>,
}

/// A field within an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_expr: TypeExpr,
    pub decorators: Vec<Decorator>,
    pub label: Option<String>,
    pub description: Option<String>,
}

impl Entity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            meta: Vec::new(),
            fields: IndexMap::new(),
            coherence_groups: IndexMap::new(),
            triggers: Vec::new(),
        }
    }

    /// Check if entity has a given entity-level decorator.
    pub fn has_meta(&self, check: &dyn Fn(&Decorator) -> bool) -> bool {
        self.meta.iter().any(check)
    }

    /// Check if entity is readonly.
    pub fn is_readonly(&self) -> bool {
        self.meta.iter().any(|d| matches!(d, Decorator::Readonly))
    }

    /// Check if entity is immutable.
    pub fn is_immutable(&self) -> bool {
        self.meta.iter().any(|d| matches!(d, Decorator::Immutable))
    }

    /// Get the primary key field, if any.
    pub fn primary_key(&self) -> Option<&Field> {
        self.fields
            .values()
            .find(|f| f.decorators.iter().any(|d| matches!(d, Decorator::Primary)))
    }
}

impl Field {
    pub fn new(name: impl Into<String>, type_expr: TypeExpr) -> Self {
        Self {
            name: name.into(),
            type_expr,
            decorators: Vec::new(),
            label: None,
            description: None,
        }
    }

    pub fn with_decorators(mut self, decorators: Vec<Decorator>) -> Self {
        self.decorators = decorators;
        self
    }

    /// Check if field has a specific decorator.
    pub fn has_decorator(&self, check: &dyn Fn(&Decorator) -> bool) -> bool {
        self.decorators.iter().any(check)
    }

    pub fn is_auto(&self) -> bool {
        self.decorators.iter().any(|d| matches!(d, Decorator::Auto))
    }

    pub fn is_primary(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::Primary))
    }

    pub fn is_unique(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::Unique | Decorator::Primary))
    }

    pub fn is_optional(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::Optional))
    }

    pub fn is_derived(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::Derived(_)))
    }

    pub fn is_computed(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::Compute(_)))
    }

    pub fn is_default_chain(&self) -> bool {
        self.decorators
            .iter()
            .any(|d| matches!(d, Decorator::DefaultChain { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrimitiveType;

    #[test]
    fn test_entity_new() {
        let e = Entity::new("User");
        assert_eq!(e.name, "User");
        assert!(e.fields.is_empty());
        assert!(e.meta.is_empty());
    }

    #[test]
    fn test_primary_key() {
        let mut e = Entity::new("User");
        e.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                .with_decorators(vec![Decorator::Primary]),
        );
        e.fields.insert(
            "name".into(),
            Field::new("name", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        let pk = e.primary_key().unwrap();
        assert_eq!(pk.name, "id");
    }

    #[test]
    fn test_field_flags() {
        let f = Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
            .with_decorators(vec![Decorator::Primary]);
        assert!(f.is_primary());
        assert!(f.is_unique());
        assert!(!f.is_auto());
        assert!(!f.is_optional());
    }
}
