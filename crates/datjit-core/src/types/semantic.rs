use serde::{Deserialize, Serialize};

/// A semantic type that carries domain meaning beyond its underlying primitive.
/// Uses dot-notation namespaces: `person.full`, `address.city`, `email`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticType {
    /// Top-level namespace: "person", "address", "email", "text", etc.
    pub namespace: String,
    /// Sub-tag within namespace: "full", "city", "". Empty for top-level like "email".
    pub tag: String,
    /// Parameters: e.g., currency("EUR"), text.paragraphs("3")
    pub params: Vec<String>,
    /// Domain scope from @domain() decorator
    pub domain: Option<String>,
}

impl SemanticType {
    pub fn new(namespace: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            tag: tag.into(),
            params: Vec::new(),
            domain: None,
        }
    }

    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }

    /// Full dotted name, e.g., "person.full" or "email"
    pub fn full_name(&self) -> String {
        if self.tag.is_empty() {
            self.namespace.clone()
        } else {
            format!("{}.{}", self.namespace, self.tag)
        }
    }

    /// Known semantic type namespaces
    pub fn is_known_namespace(name: &str) -> bool {
        matches!(
            name,
            "person"
                | "email"
                | "phone"
                | "url"
                | "ipv4"
                | "ipv6"
                | "mac"
                | "address"
                | "geo"
                | "timezone"
                | "currency"
                | "credit_card"
                | "iban"
                | "swift"
                | "text"
                | "product"
                | "company"
                | "job"
                | "color"
                | "file"
                | "sku"
                | "slug"
                | "code"
                | "hash"
        )
    }

    /// Parse a dotted name like "person.full" or "email" into a SemanticType.
    /// Returns None if the namespace is not recognized.
    pub fn parse(input: &str) -> Option<Self> {
        if let Some((ns, tag)) = input.split_once('.') {
            if Self::is_known_namespace(ns) {
                Some(Self::new(ns, tag))
            } else {
                None
            }
        } else if Self::is_known_namespace(input) {
            Some(Self::new(input, ""))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dotted() {
        let st = SemanticType::parse("person.full").unwrap();
        assert_eq!(st.namespace, "person");
        assert_eq!(st.tag, "full");
        assert_eq!(st.full_name(), "person.full");
    }

    #[test]
    fn test_parse_top_level() {
        let st = SemanticType::parse("email").unwrap();
        assert_eq!(st.namespace, "email");
        assert_eq!(st.tag, "");
        assert_eq!(st.full_name(), "email");
    }

    #[test]
    fn test_parse_unknown() {
        assert!(SemanticType::parse("foobar").is_none());
        assert!(SemanticType::parse("unknown.tag").is_none());
    }

    #[test]
    fn test_all_namespaces() {
        let names = [
            "person", "email", "phone", "url", "address", "geo", "timezone", "currency",
            "credit_card", "iban", "swift", "text", "product", "company", "job", "color", "file",
            "sku", "slug", "code", "hash", "ipv4", "ipv6", "mac",
        ];
        for name in names {
            assert!(
                SemanticType::is_known_namespace(name),
                "{name} should be known"
            );
        }
    }
}
