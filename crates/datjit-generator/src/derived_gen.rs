use indexmap::IndexMap;

use datjit_core::error::DatjitError;
use datjit_core::model::decorator::{BinaryOp, Expression, FieldPath, LiteralValue, UnaryOp};
use datjit_core::value::Value;

/// Evaluate a @derived expression against the current row and all generated data.
pub fn evaluate_derived(
    expr: &Expression,
    row: &IndexMap<String, Value>,
    all_data: &IndexMap<String, Vec<IndexMap<String, Value>>>,
) -> Result<Value, DatjitError> {
    match expr {
        Expression::Literal(lit) => Ok(literal_to_value(lit)),

        Expression::FieldRef(path) => resolve_field_ref(path, row, all_data),

        Expression::BinaryOp { left, op, right } => {
            let lval = evaluate_derived(left, row, all_data)?;
            let rval = evaluate_derived(right, row, all_data)?;
            evaluate_binary_op(&lval, op, &rval)
        }

        Expression::UnaryOp { op, operand } => {
            let val = evaluate_derived(operand, row, all_data)?;
            evaluate_unary_op(op, &val)
        }

        Expression::FunctionCall { name, args } => evaluate_function(name, args, row, all_data),

        Expression::InList { value, list } => {
            let val = evaluate_derived(value, row, all_data)?;
            for item in list {
                let item_val = evaluate_derived(item, row, all_data)?;
                if val == item_val {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }
    }
}

fn literal_to_value(lit: &LiteralValue) -> Value {
    match lit {
        LiteralValue::String(s) => Value::String(s.clone()),
        LiteralValue::Int(n) => Value::Int(*n),
        LiteralValue::Float(f) => Value::Float(*f),
        LiteralValue::Bool(b) => Value::Bool(*b),
        LiteralValue::Null => Value::Null,
    }
}

/// Resolve a field path. Single segment = look up in current row.
/// Multi-segment = traverse references in all_data.
fn resolve_field_ref(
    path: &FieldPath,
    row: &IndexMap<String, Value>,
    all_data: &IndexMap<String, Vec<IndexMap<String, Value>>>,
) -> Result<Value, DatjitError> {
    if path.segments.is_empty() {
        return Ok(Value::Null);
    }

    if path.segments.len() == 1 {
        let field_name = &path.segments[0];
        return Ok(row.get(field_name).cloned().unwrap_or(Value::Null));
    }

    // Multi-segment: e.g., "project.key"
    // First segment is a field in the current row that references another entity.
    let ref_field = &path.segments[0];
    let target_field = &path.segments[1];

    match row.get(ref_field) {
        Some(Value::Ref(entity_name, pk_value)) => {
            // Look up the referenced entity row by primary key
            if let Some(entity_rows) = all_data.get(entity_name.as_str()) {
                for entity_row in entity_rows {
                    // Check first field (assumed PK) matches
                    if let Some(first_val) = entity_row.values().next() {
                        if first_val == pk_value.as_ref() {
                            return Ok(entity_row
                                .get(target_field)
                                .cloned()
                                .unwrap_or(Value::Null));
                        }
                    }
                }
            }
            Ok(Value::Null)
        }
        _ => Ok(Value::Null),
    }
}

fn evaluate_binary_op(left: &Value, op: &BinaryOp, right: &Value) -> Result<Value, DatjitError> {
    match op {
        // Arithmetic operations
        BinaryOp::Add => numeric_op(left, right, |a, b| a + b, |a, b| a + b),
        BinaryOp::Sub => numeric_op(left, right, |a, b| a - b, |a, b| a - b),
        BinaryOp::Mul => numeric_op(left, right, |a, b| a * b, |a, b| a * b),
        BinaryOp::Div => {
            // Check for division by zero
            match (right.as_i64(), right.as_f64()) {
                (Some(0), _) => {
                    return Err(DatjitError::Generation("division by zero".into()));
                }
                (_, Some(f)) if f == 0.0 => {
                    return Err(DatjitError::Generation("division by zero".into()));
                }
                _ => {}
            }
            numeric_op(left, right, |a, b| a / b, |a, b| a / b)
        }
        BinaryOp::Mod => {
            match (right.as_i64(), right.as_f64()) {
                (Some(0), _) => {
                    return Err(DatjitError::Generation("modulo by zero".into()));
                }
                _ => {}
            }
            numeric_op(left, right, |a, b| a % b, |a, b| a % b)
        }

        // Comparison operations
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Neq => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => compare_op(left, right, |ord| ord == std::cmp::Ordering::Less),
        BinaryOp::Gt => compare_op(left, right, |ord| ord == std::cmp::Ordering::Greater),
        BinaryOp::Lte => compare_op(left, right, |ord| ord != std::cmp::Ordering::Greater),
        BinaryOp::Gte => compare_op(left, right, |ord| ord != std::cmp::Ordering::Less),

        // Logical operations
        BinaryOp::And => {
            let lb = to_bool(left);
            let rb = to_bool(right);
            Ok(Value::Bool(lb && rb))
        }
        BinaryOp::Or => {
            let lb = to_bool(left);
            let rb = to_bool(right);
            Ok(Value::Bool(lb || rb))
        }
    }
}

fn numeric_op(
    left: &Value,
    right: &Value,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
) -> Result<Value, DatjitError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
        _ => Err(DatjitError::Generation(format!(
            "cannot perform arithmetic on {:?} and {:?}",
            left, right
        ))),
    }
}

fn compare_op(
    left: &Value,
    right: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, DatjitError> {
    let ordering = match (left, right) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Date(a), Value::Date(b)) => a.cmp(b),
        (Value::DateTime(a), Value::DateTime(b)) => a.cmp(b),
        _ => {
            return Err(DatjitError::Generation(format!(
                "cannot compare {:?} and {:?}",
                left, right
            )));
        }
    };
    Ok(Value::Bool(pred(ordering)))
}

fn to_bool(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn evaluate_unary_op(op: &UnaryOp, val: &Value) -> Result<Value, DatjitError> {
    match op {
        UnaryOp::Not => Ok(Value::Bool(!to_bool(val))),
        UnaryOp::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(DatjitError::Generation(format!("cannot negate {:?}", val))),
        },
    }
}

fn evaluate_function(
    name: &str,
    args: &[Expression],
    row: &IndexMap<String, Value>,
    all_data: &IndexMap<String, Vec<IndexMap<String, Value>>>,
) -> Result<Value, DatjitError> {
    match name {
        "concat" => {
            let evaluated: Result<Vec<Value>, _> = args
                .iter()
                .map(|a| evaluate_derived(a, row, all_data))
                .collect();
            let vals = evaluated?;
            // If the last arg is a literal string that looks like a separator, use it
            // Otherwise just concatenate with empty string
            let parts: Vec<String> = vals.iter().map(|v| v.to_output_string()).collect();
            Ok(Value::String(parts.join("")))
        }

        "count" => {
            if args.is_empty() {
                return Ok(Value::Int(0));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            match val {
                Value::List(items) => Ok(Value::Int(items.len() as i64)),
                _ => {
                    // Try to interpret as entity name
                    if let Expression::FieldRef(path) = &args[0] {
                        if path.segments.len() == 1 {
                            let entity_name = &path.segments[0];
                            if let Some(rows) = all_data.get(entity_name.as_str()) {
                                return Ok(Value::Int(rows.len() as i64));
                            }
                        }
                    }
                    Ok(Value::Int(1))
                }
            }
        }

        "sum" => aggregate_function(args, row, all_data, |vals| {
            let s: f64 = vals.iter().filter_map(|v| v.as_f64()).sum();
            if vals.iter().all(|v| matches!(v, Value::Int(_))) {
                Value::Int(s as i64)
            } else {
                Value::Float(s)
            }
        }),

        "avg" => aggregate_function(args, row, all_data, |vals| {
            let nums: Vec<f64> = vals.iter().filter_map(|v| v.as_f64()).collect();
            if nums.is_empty() {
                Value::Float(0.0)
            } else {
                Value::Float(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        }),

        "min" => aggregate_function(args, row, all_data, |vals| {
            vals.iter()
                .filter_map(|v| v.as_f64())
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|f| {
                    if vals.iter().all(|v| matches!(v, Value::Int(_))) {
                        Value::Int(f as i64)
                    } else {
                        Value::Float(f)
                    }
                })
                .unwrap_or(Value::Null)
        }),

        "max" => aggregate_function(args, row, all_data, |vals| {
            vals.iter()
                .filter_map(|v| v.as_f64())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|f| {
                    if vals.iter().all(|v| matches!(v, Value::Int(_))) {
                        Value::Int(f as i64)
                    } else {
                        Value::Float(f)
                    }
                })
                .unwrap_or(Value::Null)
        }),

        "years_since" => {
            if args.is_empty() {
                return Ok(Value::Int(0));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            let date_str = val.to_output_string();
            let years = compute_years_since(&date_str);
            Ok(Value::Int(years))
        }

        "days_between" => {
            if args.len() < 2 {
                return Ok(Value::Int(0));
            }
            let a = evaluate_derived(&args[0], row, all_data)?;
            let b = evaluate_derived(&args[1], row, all_data)?;
            let days = compute_days_between(&a.to_output_string(), &b.to_output_string());
            Ok(Value::Int(days))
        }

        "if" => {
            if args.len() < 3 {
                return Err(DatjitError::Generation(
                    "if() requires 3 arguments: condition, then, else".into(),
                ));
            }
            let cond = evaluate_derived(&args[0], row, all_data)?;
            if to_bool(&cond) {
                evaluate_derived(&args[1], row, all_data)
            } else {
                evaluate_derived(&args[2], row, all_data)
            }
        }

        "round" => {
            if args.is_empty() {
                return Ok(Value::Float(0.0));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            let decimals = if args.len() > 1 {
                evaluate_derived(&args[1], row, all_data)?
                    .as_i64()
                    .unwrap_or(0)
            } else {
                0
            };
            match val.as_f64() {
                Some(f) => {
                    let factor = 10f64.powi(decimals as i32);
                    Ok(Value::Float((f * factor).round() / factor))
                }
                None => Ok(val),
            }
        }

        "lower" => {
            if args.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            Ok(Value::String(val.to_output_string().to_lowercase()))
        }

        "upper" => {
            if args.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            Ok(Value::String(val.to_output_string().to_uppercase()))
        }

        "slug" => {
            if args.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            let s = val.to_output_string();
            let slug = slugify(&s);
            Ok(Value::String(slug))
        }

        "starts_with" => {
            if args.len() < 2 {
                return Err(DatjitError::Generation(
                    "starts_with() requires 2 arguments".into(),
                ));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            let prefix = evaluate_derived(&args[1], row, all_data)?;
            let s = val.to_output_string();
            let p = prefix.to_output_string();
            Ok(Value::Bool(s.starts_with(&p)))
        }

        "ends_with" => {
            if args.len() < 2 {
                return Err(DatjitError::Generation(
                    "ends_with() requires 2 arguments".into(),
                ));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            let suffix = evaluate_derived(&args[1], row, all_data)?;
            let s = val.to_output_string();
            let sfx = suffix.to_output_string();
            Ok(Value::Bool(s.ends_with(&sfx)))
        }

        "all_equal" => {
            // Used in cross-row checks: all_equal(field) checks if a list of values are all equal.
            // When called with a single field ref, returns true (single value is trivially equal).
            if args.is_empty() {
                return Ok(Value::Bool(true));
            }
            let val = evaluate_derived(&args[0], row, all_data)?;
            match val {
                Value::List(items) => {
                    if items.is_empty() {
                        Ok(Value::Bool(true))
                    } else {
                        let first = &items[0];
                        Ok(Value::Bool(items.iter().all(|v| v == first)))
                    }
                }
                _ => Ok(Value::Bool(true)),
            }
        }

        _ => Err(DatjitError::Generation(format!(
            "unknown function: {}",
            name
        ))),
    }
}

/// Aggregate over a collection. Handles "entity.field" paths.
fn aggregate_function(
    args: &[Expression],
    row: &IndexMap<String, Value>,
    all_data: &IndexMap<String, Vec<IndexMap<String, Value>>>,
    agg: impl Fn(&[Value]) -> Value,
) -> Result<Value, DatjitError> {
    if args.is_empty() {
        return Ok(agg(&[]));
    }

    // Check if the argument is a FieldRef with two segments: "entity.field"
    if let Expression::FieldRef(path) = &args[0] {
        if path.segments.len() == 2 {
            let entity_name = &path.segments[0];
            let field_name = &path.segments[1];
            if let Some(rows) = all_data.get(entity_name.as_str()) {
                let values: Vec<Value> = rows
                    .iter()
                    .filter_map(|r| r.get(field_name).cloned())
                    .collect();
                return Ok(agg(&values));
            }
        }
    }

    // Single value fallback
    let val = evaluate_derived(&args[0], row, all_data)?;
    match val {
        Value::List(items) => Ok(agg(&items)),
        other => Ok(agg(&[other])),
    }
}

fn compute_years_since(date_str: &str) -> i64 {
    use chrono::NaiveDate;
    let today = chrono::Utc::now().date_naive();
    // Try parsing as date or datetime
    if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let years = today.years_since(d).unwrap_or(0) as i64;
        return years;
    }
    // Try parsing datetime prefix
    if date_str.len() >= 10 {
        if let Ok(d) = NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d") {
            let years = today.years_since(d).unwrap_or(0) as i64;
            return years;
        }
    }
    0
}

fn compute_days_between(a: &str, b: &str) -> i64 {
    use chrono::NaiveDate;
    let parse = |s: &str| -> Option<NaiveDate> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok().or_else(|| {
            if s.len() >= 10 {
                NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
            } else {
                None
            }
        })
    };
    match (parse(a), parse(b)) {
        (Some(da), Some(db)) => (db - da).num_days().abs(),
        _ => 0,
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::model::decorator::{BinaryOp, FieldPath, LiteralValue, UnaryOp};

    fn empty_all_data() -> IndexMap<String, Vec<IndexMap<String, Value>>> {
        IndexMap::new()
    }

    fn make_row() -> IndexMap<String, Value> {
        let mut row = IndexMap::new();
        row.insert("qty".into(), Value::Int(5));
        row.insert("price".into(), Value::Float(10.50));
        row.insert("name".into(), Value::String("Hello World".into()));
        row.insert("active".into(), Value::Bool(true));
        row.insert("dob".into(), Value::Date("2000-01-15".into()));
        row
    }

    #[test]
    fn test_field_reference() {
        let row = make_row();
        let expr = Expression::FieldRef(FieldPath::parse("qty"));
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_arithmetic_multiply() {
        let row = make_row();
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
            op: BinaryOp::Mul,
            right: Box::new(Expression::FieldRef(FieldPath::parse("price"))),
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Float(52.5));
    }

    #[test]
    fn test_arithmetic_add_ints() {
        let row = make_row();
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
            op: BinaryOp::Add,
            right: Box::new(Expression::Literal(LiteralValue::Int(3))),
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_concat_function() {
        let row = make_row();
        let expr = Expression::FunctionCall {
            name: "concat".into(),
            args: vec![
                Expression::Literal(LiteralValue::String("Item: ".into())),
                Expression::FieldRef(FieldPath::parse("name")),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::String("Item: Hello World".into()));
    }

    #[test]
    fn test_count_entity_rows() {
        let row = make_row();
        let mut all_data = IndexMap::new();
        let mut order_rows = Vec::new();
        for i in 0..3 {
            let mut r = IndexMap::new();
            r.insert("id".into(), Value::Int(i));
            r.insert("amount".into(), Value::Float(100.0 + i as f64));
            order_rows.push(r);
        }
        all_data.insert("Order".into(), order_rows);

        let expr = Expression::FunctionCall {
            name: "count".into(),
            args: vec![Expression::FieldRef(FieldPath::parse("Order"))],
        };
        let result = evaluate_derived(&expr, &row, &all_data).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_conditional_if() {
        let row = make_row();
        let expr = Expression::FunctionCall {
            name: "if".into(),
            args: vec![
                Expression::BinaryOp {
                    left: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
                    op: BinaryOp::Gt,
                    right: Box::new(Expression::Literal(LiteralValue::Int(3))),
                },
                Expression::Literal(LiteralValue::String("high".into())),
                Expression::Literal(LiteralValue::String("low".into())),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::String("high".into()));
    }

    #[test]
    fn test_slug_function() {
        let row = make_row();
        let expr = Expression::FunctionCall {
            name: "slug".into(),
            args: vec![Expression::FieldRef(FieldPath::parse("name"))],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::String("hello-world".into()));
    }

    #[test]
    fn test_lower_upper() {
        let row = make_row();
        let lower_expr = Expression::FunctionCall {
            name: "lower".into(),
            args: vec![Expression::FieldRef(FieldPath::parse("name"))],
        };
        let upper_expr = Expression::FunctionCall {
            name: "upper".into(),
            args: vec![Expression::FieldRef(FieldPath::parse("name"))],
        };
        assert_eq!(
            evaluate_derived(&lower_expr, &row, &empty_all_data()).unwrap(),
            Value::String("hello world".into())
        );
        assert_eq!(
            evaluate_derived(&upper_expr, &row, &empty_all_data()).unwrap(),
            Value::String("HELLO WORLD".into())
        );
    }

    #[test]
    fn test_unary_not() {
        let row = make_row();
        let expr = Expression::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(Expression::FieldRef(FieldPath::parse("active"))),
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_neg() {
        let row = make_row();
        let expr = Expression::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Int(-5));
    }

    #[test]
    fn test_in_list() {
        let row = make_row();
        let expr = Expression::InList {
            value: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
            list: vec![
                Expression::Literal(LiteralValue::Int(3)),
                Expression::Literal(LiteralValue::Int(5)),
                Expression::Literal(LiteralValue::Int(7)),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_round_function() {
        let row = make_row();
        let expr = Expression::FunctionCall {
            name: "round".into(),
            args: vec![
                Expression::FieldRef(FieldPath::parse("price")),
                Expression::Literal(LiteralValue::Int(1)),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Float(10.5));
    }

    #[test]
    fn test_sum_aggregate() {
        let row = IndexMap::new();
        let mut all_data = IndexMap::new();
        let mut order_rows = Vec::new();
        for i in 1..=4 {
            let mut r = IndexMap::new();
            r.insert("id".into(), Value::Int(i));
            r.insert("amount".into(), Value::Float(i as f64 * 10.0));
            order_rows.push(r);
        }
        all_data.insert("Order".into(), order_rows);

        let expr = Expression::FunctionCall {
            name: "sum".into(),
            args: vec![Expression::FieldRef(FieldPath::parse("Order.amount"))],
        };
        let result = evaluate_derived(&expr, &row, &all_data).unwrap();
        assert_eq!(result, Value::Float(100.0));
    }

    #[test]
    fn test_days_between() {
        let mut row = IndexMap::new();
        row.insert("start".into(), Value::Date("2025-01-01".into()));
        row.insert("end".into(), Value::Date("2025-01-11".into()));

        let expr = Expression::FunctionCall {
            name: "days_between".into(),
            args: vec![
                Expression::FieldRef(FieldPath::parse("start")),
                Expression::FieldRef(FieldPath::parse("end")),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_starts_with() {
        let mut row = IndexMap::new();
        row.insert("glacct".into(), Value::String("154-3200".into()));

        let expr = Expression::FunctionCall {
            name: "starts_with".into(),
            args: vec![
                Expression::FieldRef(FieldPath::parse("glacct")),
                Expression::Literal(LiteralValue::String("154".into())),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let expr2 = Expression::FunctionCall {
            name: "starts_with".into(),
            args: vec![
                Expression::FieldRef(FieldPath::parse("glacct")),
                Expression::Literal(LiteralValue::String("200".into())),
            ],
        };
        let result2 = evaluate_derived(&expr2, &row, &empty_all_data()).unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_ends_with() {
        let mut row = IndexMap::new();
        row.insert("code".into(), Value::String("WO-L12345".into()));

        let expr = Expression::FunctionCall {
            name: "ends_with".into(),
            args: vec![
                Expression::FieldRef(FieldPath::parse("code")),
                Expression::Literal(LiteralValue::String("12345".into())),
            ],
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_all_equal() {
        let row = IndexMap::new();
        let all_data = empty_all_data();

        // Equal list
        let expr = Expression::FunctionCall {
            name: "all_equal".into(),
            args: vec![Expression::Literal(LiteralValue::Null)],
        };
        let result = evaluate_derived(&expr, &row, &all_data).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_division_by_zero() {
        let row = make_row();
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::FieldRef(FieldPath::parse("qty"))),
            op: BinaryOp::Div,
            right: Box::new(Expression::Literal(LiteralValue::Int(0))),
        };
        let result = evaluate_derived(&expr, &row, &empty_all_data());
        assert!(result.is_err());
    }
}
