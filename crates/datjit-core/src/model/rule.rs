use serde::{Deserialize, Serialize};

use super::decorator::FieldPath;

/// A cross-entity rule from the `rules:` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub expression: RuleExpression,
    pub modifier: RuleModifier,
    /// Optional user-facing error message when the rule is violated.
    pub message: Option<String>,
}

/// The parsed expression of a rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleExpression {
    /// `A.field op B.field` or `A.field op literal`
    Comparison {
        left: FieldPath,
        op: CompOp,
        right: RuleOperand,
    },
    /// `if condition then consequent`
    Conditional {
        condition: Box<RuleExpression>,
        then: Box<RuleExpression>,
    },
    /// `sum(Entity.collection.field) op value`
    Aggregate {
        func: AggFunc,
        path: FieldPath,
        op: CompOp,
        value: RuleOperand,
    },
    /// `unique(Entity.field1, Entity.field2)`
    UniqueComposite(Vec<FieldPath>),
    /// `count(Entity.collection) op value`
    CountConstraint {
        path: FieldPath,
        op: CompOp,
        value: RuleOperand,
    },
    /// Cross-row validation that runs after all rows of an entity are generated.
    CrossRow {
        entity: String,
        group_by: Option<String>,
        check: super::decorator::Expression,
        on_violation: Option<ViolationAction>,
        /// Probability of intentionally generating violations (for test data).
        probability: Option<f64>,
    },
}

/// Action to take when a cross-row rule is violated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViolationAction {
    /// Fields to set on violation, e.g. `PR.x1aeplegalerror = 1`.
    pub set_fields: Vec<(FieldPath, super::decorator::Expression)>,
    /// User-facing error message template.
    pub error: Option<String>,
}

/// The right-hand side of a rule comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleOperand {
    FieldPath(FieldPath),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Range(i64, i64),
}

/// Comparison operator.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompOp {
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    In,
}

/// Aggregate function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggFunc {
    Sum,
    Count,
    Avg,
    Min,
    Max,
}

/// Rule enforcement modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleModifier {
    /// Must always hold (default)
    Strict,
    /// Holds with probability p
    Probability(f64),
    /// Log warning if violated
    Warn,
}

impl Default for RuleModifier {
    fn default() -> Self {
        Self::Strict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_modifier_default() {
        let m = RuleModifier::default();
        assert!(matches!(m, RuleModifier::Strict));
    }

    #[test]
    fn test_comparison_rule() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Order.shipped_at"),
                op: CompOp::Gt,
                right: RuleOperand::FieldPath(FieldPath::parse("Order.placed_at")),
            },
            modifier: RuleModifier::Strict,
            message: None,
        };
        assert!(matches!(rule.expression, RuleExpression::Comparison { .. }));
    }
}
