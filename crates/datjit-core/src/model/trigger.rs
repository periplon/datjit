use serde::{Deserialize, Serialize};

/// A trigger that fires when specified fields change, causing
/// other fields to be recomputed or rules to be validated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    /// Field name(s) whose change fires this trigger.
    pub on: Vec<String>,
    /// Fields to recompute when triggered.
    pub recompute: Vec<String>,
    /// Rule names to validate when triggered.
    pub validate: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_basic() {
        let t = Trigger {
            on: vec!["wo".into()],
            recompute: vec!["pcbu".into(), "project_id".into()],
            validate: vec![],
        };
        assert_eq!(t.on, vec!["wo"]);
        assert_eq!(t.recompute.len(), 2);
        assert!(t.validate.is_empty());
    }
}
