use serde::{Deserialize, Serialize};

/// A named enum definition from the `enums:` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

/// A single enum variant, optionally with a label, weight, and description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub value: String,
    pub label: Option<String>,
    pub weight: Option<f64>,
    pub description: Option<String>,
}

impl EnumVariant {
    pub fn simple(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            weight: None,
            description: None,
        }
    }

    pub fn weighted(value: impl Into<String>, label: impl Into<String>, weight: f64) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
            weight: Some(weight),
            description: None,
        }
    }
}

impl EnumDef {
    pub fn simple(name: impl Into<String>, values: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            variants: values.into_iter().map(EnumVariant::simple).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_enum() {
        let e = EnumDef::simple("Priority", vec!["critical", "high", "medium", "low"]);
        assert_eq!(e.name, "Priority");
        assert_eq!(e.variants.len(), 4);
        assert_eq!(e.variants[0].value, "critical");
        assert!(e.variants[0].weight.is_none());
    }

    #[test]
    fn test_weighted_variant() {
        let v = EnumVariant::weighted("NA", "North America", 25.0);
        assert_eq!(v.value, "NA");
        assert_eq!(v.label, Some("North America".into()));
        assert_eq!(v.weight, Some(25.0));
    }
}
