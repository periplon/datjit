use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::entity::Entity;
use super::enum_def::EnumDef;
use super::mcp_tool::McpToolDef;
use super::rule::Rule;

/// The top-level DDL document representing an entire parsed schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DdlDocument {
    /// Domain identifier (required)
    pub domain: String,
    /// Schema version (optional)
    pub version: Option<String>,
    /// Deterministic generation seed (optional)
    pub seed: Option<u64>,
    /// Default locale (default: "en-US")
    pub locale: String,
    /// Volume configuration per entity
    pub volume: HashMap<String, VolumeSpec>,
    /// Generation configuration
    pub generation: GenerationConfig,
    /// Entity definitions in definition order
    pub entities: IndexMap<String, Entity>,
    /// Named enum definitions
    pub enums: HashMap<String, EnumDef>,
    /// Reusable type definitions
    pub types: HashMap<String, TypeDef>,
    /// Cross-entity rules
    pub rules: Vec<Rule>,
    /// Tool overrides per entity
    pub tools: HashMap<String, ToolOverrides>,
    /// MCP tool declarations
    pub mcp_tools: IndexMap<String, McpToolDef>,
}

/// Volume specification for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VolumeSpec {
    Exact(usize),
    Range(usize, usize),
    Inferred, // ~ in the spec: inferred from relationships
}

/// Generation configuration block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub seed: Option<u64>,
    pub locale: String,
    pub locales: Option<HashMap<String, u32>>,
    pub null_strategy: NullStrategy,
    pub id_format: IdFormat,
    pub date_format: String,
    pub currency_format: CurrencyFormat,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            seed: None,
            locale: "en-US".into(),
            locales: None,
            null_strategy: NullStrategy::Realistic,
            id_format: IdFormat::Uuid,
            date_format: "iso8601".into(),
            currency_format: CurrencyFormat::Decimal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NullStrategy {
    Realistic,
    Never,
    Sparse,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdFormat {
    Uuid,
    Sequential,
    Cuid,
    Ulid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyFormat {
    Decimal,
    IntegerCents,
}

/// A reusable compound type definition from the `types:` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub fields: IndexMap<String, crate::model::entity::Field>,
}

/// Tool overrides for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolOverrides {
    pub list: Option<ListOverride>,
    pub create: Option<MutationOverride>,
    pub update: Option<MutationOverride>,
    pub delete: Option<DeleteOverride>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListOverride {
    pub filters: Option<Vec<String>>,
    pub sorts: Option<Vec<String>>,
    pub page_size: Option<usize>,
    pub max_page_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationOverride {
    pub disabled: bool,
    pub required: Option<Vec<String>>,
    pub optional: Option<Vec<String>>,
    pub mutable: Option<Vec<String>>,
    pub immutable: Option<Vec<String>>,
    pub defaults: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteOverride {
    pub disabled: bool,
    pub strategy: Option<DeleteStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeleteStrategy {
    Soft,
    Hard,
}

impl DdlDocument {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            version: None,
            seed: None,
            locale: "en-US".into(),
            volume: HashMap::new(),
            generation: GenerationConfig::default(),
            entities: IndexMap::new(),
            enums: HashMap::new(),
            types: HashMap::new(),
            rules: Vec::new(),
            tools: HashMap::new(),
            mcp_tools: IndexMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = DdlDocument::new("test_domain");
        assert_eq!(doc.domain, "test_domain");
        assert_eq!(doc.locale, "en-US");
        assert!(doc.entities.is_empty());
        assert!(doc.seed.is_none());
    }

    #[test]
    fn test_generation_config_default() {
        let cfg = GenerationConfig::default();
        assert_eq!(cfg.locale, "en-US");
        assert!(matches!(cfg.null_strategy, NullStrategy::Realistic));
        assert!(matches!(cfg.id_format, IdFormat::Uuid));
    }

    #[test]
    fn test_volume_spec() {
        let exact = VolumeSpec::Exact(1000);
        assert!(matches!(exact, VolumeSpec::Exact(1000)));

        let range = VolumeSpec::Range(100, 200);
        assert!(matches!(range, VolumeSpec::Range(100, 200)));
    }
}
