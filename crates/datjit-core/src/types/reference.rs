use serde::{Deserialize, Serialize};

/// Reference types that express relationships between entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceType {
    /// `->Entity` or `->Entity?` — foreign key (belongs-to)
    BelongsTo { target: String, optional: bool },
    /// `->self` or `->self?` — self-referential
    SelfRef { optional: bool },
    /// `[Entity]` — has-many (reverse side)
    HasMany { target: String },
    /// `<->Entity` — bidirectional many-to-many
    ManyToMany { target: String },
    /// `->Post | ->Photo | ->Video` — polymorphic reference
    Polymorphic { targets: Vec<String> },
}

impl ReferenceType {
    /// Get the target entity name(s) for this reference.
    pub fn targets(&self) -> Vec<&str> {
        match self {
            ReferenceType::BelongsTo { target, .. } => vec![target.as_str()],
            ReferenceType::SelfRef { .. } => vec![],
            ReferenceType::HasMany { target } => vec![target.as_str()],
            ReferenceType::ManyToMany { target } => vec![target.as_str()],
            ReferenceType::Polymorphic { targets } => targets.iter().map(|s| s.as_str()).collect(),
        }
    }

    /// Whether this reference is optional (can be null).
    pub fn is_optional(&self) -> bool {
        match self {
            ReferenceType::BelongsTo { optional, .. } => *optional,
            ReferenceType::SelfRef { optional } => *optional,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belongs_to() {
        let r = ReferenceType::BelongsTo {
            target: "User".into(),
            optional: false,
        };
        assert_eq!(r.targets(), vec!["User"]);
        assert!(!r.is_optional());
    }

    #[test]
    fn test_optional_belongs_to() {
        let r = ReferenceType::BelongsTo {
            target: "User".into(),
            optional: true,
        };
        assert!(r.is_optional());
    }

    #[test]
    fn test_self_ref() {
        let r = ReferenceType::SelfRef { optional: true };
        assert!(r.targets().is_empty());
        assert!(r.is_optional());
    }

    #[test]
    fn test_polymorphic() {
        let r = ReferenceType::Polymorphic {
            targets: vec!["Post".into(), "Photo".into(), "Video".into()],
        };
        assert_eq!(r.targets(), vec!["Post", "Photo", "Video"]);
    }
}
