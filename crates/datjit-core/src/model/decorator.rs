use serde::{Deserialize, Serialize};

/// A decorator annotation that modifies field or entity behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decorator {
    // Identity & Uniqueness
    Auto,
    Unique,
    Primary,
    Index,
    Immutable,

    // Value Constraints
    Range(RangeValue, RangeValue),
    Min(RangeValue),
    Max(RangeValue),
    Len(usize, usize),
    Pattern(PatternKind),
    Values(Vec<String>),
    NotEmpty,
    Optional,
    Default(LiteralValue),

    // Distribution Hints
    Dist(Distribution),
    NullRate(f64),

    // Relational
    Count(CountSpec),
    From(Vec<String>),
    After(FieldPath),
    Before(FieldPath),
    Within(DurationLiteral, FieldPath),
    Correlated(String, f64),

    // Derivation
    Derived(Expression),

    // Tool Behavior
    Readonly,
    NoDelete,
    SoftDelete,
    Sortable,
    Filterable,
    Searchable,
    Hidden,
    Sensitive,
    Paginated(usize),

    // Relationship Behavior
    Cascade,
    Restrict,
    SetNull,
    Eager,
    Lazy,

    // Entity-Level
    Timestamps,
    Versioned,
    Cacheable(u64),

    // Scoping
    Domain(String),
    Locale(String),
}

/// A range boundary value — can be numeric, date, or relative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RangeValue {
    Int(i64),
    Float(f64),
    Date(String),
    Now,
    Relative(String), // e.g., "now-90d", "now+3m", "now-1y"
}

/// Pattern kind — regex or template.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternKind {
    Regex(String),
    Template(String),
}

/// Count specification for relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CountSpec {
    Exact(usize),
    Range(usize, usize),
}

/// A literal value for @default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// Statistical distribution for data generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Distribution {
    Uniform,
    Normal { mu: f64, sigma: f64 },
    LogNormal { mu: f64, sigma: f64 },
    Exponential { lambda: f64 },
    Geometric { p: f64 },
    Zipf { s: f64 },
    Bimodal { peaks: (f64, f64) },
    Categorical(Vec<f64>),
    Weighted(Vec<(String, f64)>),
}

/// A field path like "Order.shipped_at" or "project.key".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldPath {
    pub segments: Vec<String>,
}

impl FieldPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn parse(input: &str) -> Self {
        Self {
            segments: input.split('.').map(String::from).collect(),
        }
    }
}

/// Duration literal like "90d", "1y", "3m".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DurationLiteral {
    pub value: String,
}

/// Expression for @derived fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(LiteralValue),
    FieldRef(FieldPath),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    InList {
        value: Box<Expression>,
        list: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_path_parse() {
        let fp = FieldPath::parse("Order.shipped_at");
        assert_eq!(fp.segments, vec!["Order", "shipped_at"]);
    }

    #[test]
    fn test_distribution_categorical() {
        let d = Distribution::Categorical(vec![70.0, 25.0, 5.0]);
        match d {
            Distribution::Categorical(probs) => {
                assert_eq!(probs.len(), 3);
                assert!((probs.iter().sum::<f64>() - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Categorical"),
        }
    }

    #[test]
    fn test_count_spec() {
        assert_eq!(CountSpec::Exact(5), CountSpec::Exact(5));
        assert_eq!(CountSpec::Range(0, 20), CountSpec::Range(0, 20));
    }

    #[test]
    fn test_expression_binary() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
            op: BinaryOp::Mul,
            right: Box::new(Expression::FieldRef(FieldPath::parse("product.price"))),
        };
        assert!(matches!(expr, Expression::BinaryOp { .. }));
    }
}
