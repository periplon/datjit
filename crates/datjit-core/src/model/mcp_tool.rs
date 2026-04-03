use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Declaration of an external MCP tool in the DDL schema.
/// These are exported via tool inference but never called during generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    /// Input parameter names and their type strings.
    pub input: IndexMap<String, String>,
    /// Output field names and their type strings.
    pub output: IndexMap<String, String>,
    pub kind: McpToolKind,
}

/// The kind of MCP tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpToolKind {
    Lookup,
    Validation,
    Dropdown,
    Action,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_def() {
        let tool = McpToolDef {
            name: "load_default_pcbu".into(),
            description: "Load default PCBU".into(),
            input: {
                let mut m = IndexMap::new();
                m.insert("wo_id".into(), "string".into());
                m
            },
            output: {
                let mut m = IndexMap::new();
                m.insert("pcbu".into(), "string".into());
                m
            },
            kind: McpToolKind::Lookup,
        };
        assert_eq!(tool.kind, McpToolKind::Lookup);
        assert_eq!(tool.input.len(), 1);
    }
}
