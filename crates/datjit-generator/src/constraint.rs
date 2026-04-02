use datjit_core::error::DatjitError;
use datjit_core::model::decorator::FieldPath;
use datjit_core::model::rule::{CompOp, Rule, RuleExpression, RuleModifier, RuleOperand};
use datjit_core::value::Value;
use indexmap::IndexMap;
use rand::Rng;

pub type Row = IndexMap<String, Value>;

/// Check if a rule applies to the given entity.
pub fn rule_applies_to_entity(rule: &Rule, entity_name: &str) -> bool {
    match &rule.expression {
        RuleExpression::Comparison { left, .. } => field_path_starts_with(left, entity_name),
        RuleExpression::Conditional { condition, .. } => {
            expr_applies_to_entity(condition, entity_name)
        }
        RuleExpression::Aggregate { path, .. } => field_path_starts_with(path, entity_name),
        RuleExpression::UniqueComposite(paths) => {
            paths.iter().any(|p| field_path_starts_with(p, entity_name))
        }
        RuleExpression::CountConstraint { path, .. } => {
            field_path_starts_with(path, entity_name)
        }
    }
}

fn expr_applies_to_entity(expr: &RuleExpression, entity_name: &str) -> bool {
    match expr {
        RuleExpression::Comparison { left, .. } => field_path_starts_with(left, entity_name),
        RuleExpression::Conditional { condition, .. } => {
            expr_applies_to_entity(condition, entity_name)
        }
        RuleExpression::Aggregate { path, .. } => field_path_starts_with(path, entity_name),
        RuleExpression::UniqueComposite(paths) => {
            paths.iter().any(|p| field_path_starts_with(p, entity_name))
        }
        RuleExpression::CountConstraint { path, .. } => {
            field_path_starts_with(path, entity_name)
        }
    }
}

fn field_path_starts_with(path: &FieldPath, entity_name: &str) -> bool {
    path.segments.first().map(|s| s.as_str()) == Some(entity_name)
}

/// Evaluate a rule against a generated row. Returns true if the rule is satisfied.
pub fn evaluate_rule(
    rule: &Rule,
    entity_name: &str,
    row: &Row,
    all_data: &IndexMap<String, Vec<Row>>,
) -> bool {
    evaluate_expression(&rule.expression, entity_name, row, all_data)
}

fn evaluate_expression(
    expr: &RuleExpression,
    entity_name: &str,
    row: &Row,
    all_data: &IndexMap<String, Vec<Row>>,
) -> bool {
    match expr {
        RuleExpression::Comparison { left, op, right } => {
            let left_val = resolve_field_path(left, entity_name, row, all_data);
            let right_val = resolve_operand(right, entity_name, row, all_data);
            match (left_val, right_val) {
                (Some(l), Some(r)) => compare_values(&l, op, &r),
                _ => false,
            }
        }
        RuleExpression::Conditional { condition, then } => {
            let cond_met = evaluate_expression(condition, entity_name, row, all_data);
            if cond_met {
                evaluate_expression(then, entity_name, row, all_data)
            } else {
                // If condition is not met, the rule is vacuously satisfied
                true
            }
        }
        // Aggregate and UniqueComposite/CountConstraint are complex cross-entity checks;
        // for now, pass them (they'd need full dataset analysis).
        RuleExpression::Aggregate { .. }
        | RuleExpression::UniqueComposite(_)
        | RuleExpression::CountConstraint { .. } => true,
    }
}

/// Resolve a FieldPath to a Value.
/// For "Entity.field", strip the entity prefix and look up in the row.
/// For "Entity.ref_field.nested_field", follow the reference.
fn resolve_field_path(
    path: &FieldPath,
    entity_name: &str,
    row: &Row,
    all_data: &IndexMap<String, Vec<Row>>,
) -> Option<Value> {
    if path.segments.is_empty() {
        return None;
    }

    let (start_entity, field_segments) = if path.segments[0] == entity_name {
        (entity_name, &path.segments[1..])
    } else {
        // Cross-entity path — the first segment is a different entity
        return resolve_cross_entity_path(path, all_data);
    };

    if field_segments.is_empty() {
        return None;
    }

    let first_field = &field_segments[0];
    let value = row.get(first_field)?;

    if field_segments.len() == 1 {
        return Some(value.clone());
    }

    // Follow references: if value is Ref(target_entity, pk), look up in all_data
    if let Value::Ref(target_entity, pk) = value {
        if let Some(target_rows) = all_data.get(target_entity.as_str()) {
            // Find the row with matching primary key
            for target_row in target_rows {
                if let Some(target_pk) = target_row.values().next() {
                    if target_pk == pk.as_ref() {
                        // Recurse with remaining segments
                        let remaining = FieldPath::new(
                            std::iter::once(target_entity.clone())
                                .chain(field_segments[1..].iter().cloned())
                                .collect(),
                        );
                        return resolve_field_path(
                            &remaining,
                            target_entity,
                            target_row,
                            all_data,
                        );
                    }
                }
            }
        }
    }

    let _ = start_entity;
    None
}

fn resolve_cross_entity_path(
    path: &FieldPath,
    all_data: &IndexMap<String, Vec<Row>>,
) -> Option<Value> {
    if path.segments.len() < 2 {
        return None;
    }
    let entity = &path.segments[0];
    let field = &path.segments[1];
    // Return the value from the last row of that entity (best effort)
    let rows = all_data.get(entity.as_str())?;
    let last_row = rows.last()?;
    last_row.get(field).cloned()
}

fn resolve_operand(
    operand: &RuleOperand,
    entity_name: &str,
    row: &Row,
    all_data: &IndexMap<String, Vec<Row>>,
) -> Option<Value> {
    match operand {
        RuleOperand::FieldPath(fp) => resolve_field_path(fp, entity_name, row, all_data),
        RuleOperand::Int(n) => Some(Value::Int(*n)),
        RuleOperand::Float(f) => Some(Value::Float(*f)),
        RuleOperand::String(s) => Some(Value::String(s.clone())),
        RuleOperand::Bool(b) => Some(Value::Bool(*b)),
        RuleOperand::Null => Some(Value::Null),
        RuleOperand::Range(_, _) => None, // Ranges are not directly comparable as values
    }
}

fn compare_values(left: &Value, op: &CompOp, right: &Value) -> bool {
    match op {
        CompOp::Eq => left == right,
        CompOp::Neq => left != right,
        CompOp::Gt => numeric_cmp(left, right).map_or(false, |o| o == std::cmp::Ordering::Greater),
        CompOp::Lt => numeric_cmp(left, right).map_or(false, |o| o == std::cmp::Ordering::Less),
        CompOp::Gte => {
            numeric_cmp(left, right).map_or(false, |o| o != std::cmp::Ordering::Less)
        }
        CompOp::Lte => {
            numeric_cmp(left, right).map_or(false, |o| o != std::cmp::Ordering::Greater)
        }
        CompOp::In => {
            // "In" checks if left value is contained in right (if right is a list-like concept)
            left == right
        }
    }
}

fn numeric_cmp(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        (Value::DateTime(a), Value::DateTime(b)) => Some(a.cmp(b)),
        (Value::Date(a), Value::Date(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

/// Enforce all applicable rules for a row. For @strict rules, return error if violated.
/// For @probability rules, only enforce with the given probability.
/// For @warn rules, just log and return Ok.
pub fn enforce_rules(
    rules: &[Rule],
    entity_name: &str,
    row: &Row,
    all_data: &IndexMap<String, Vec<Row>>,
    rng: &mut impl Rng,
) -> Result<(), DatjitError> {
    for rule in rules {
        if !rule_applies_to_entity(rule, entity_name) {
            continue;
        }

        match &rule.modifier {
            RuleModifier::Strict => {
                if !evaluate_rule(rule, entity_name, row, all_data) {
                    return Err(DatjitError::ConstraintViolation(format!(
                        "Strict rule violated for entity '{entity_name}': {rule:?}"
                    )));
                }
            }
            RuleModifier::Probability(p) => {
                if rng.gen_bool(p.clamp(0.0, 1.0)) {
                    if !evaluate_rule(rule, entity_name, row, all_data) {
                        return Err(DatjitError::ConstraintViolation(format!(
                            "Probability rule violated for entity '{entity_name}': {rule:?}"
                        )));
                    }
                }
            }
            RuleModifier::Warn => {
                if !evaluate_rule(rule, entity_name, row, all_data) {
                    eprintln!(
                        "Warning: rule violated for entity '{entity_name}': {rule:?}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::model::rule::{CompOp, Rule, RuleExpression, RuleModifier, RuleOperand};
    use datjit_core::model::decorator::FieldPath;

    fn make_row(pairs: Vec<(&str, Value)>) -> Row {
        pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    #[test]
    fn test_simple_comparison_satisfied() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.status"),
                op: CompOp::Eq,
                right: RuleOperand::String("done".into()),
            },
            modifier: RuleModifier::Strict,
        };

        let row = make_row(vec![("status", Value::String("done".into()))]);
        let all_data = IndexMap::new();

        assert!(evaluate_rule(&rule, "Task", &row, &all_data));
    }

    #[test]
    fn test_simple_comparison_violated() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.status"),
                op: CompOp::Eq,
                right: RuleOperand::String("done".into()),
            },
            modifier: RuleModifier::Strict,
        };

        let row = make_row(vec![("status", Value::String("pending".into()))]);
        let all_data = IndexMap::new();

        assert!(!evaluate_rule(&rule, "Task", &row, &all_data));
    }

    #[test]
    fn test_numeric_gt_comparison() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Order.total"),
                op: CompOp::Gt,
                right: RuleOperand::Int(0),
            },
            modifier: RuleModifier::Strict,
        };

        let row = make_row(vec![("total", Value::Int(42))]);
        let all_data = IndexMap::new();

        assert!(evaluate_rule(&rule, "Order", &row, &all_data));
    }

    #[test]
    fn test_conditional_rule() {
        // if Task.status == "shipped" then Task.shipped_at != null
        let rule = Rule {
            expression: RuleExpression::Conditional {
                condition: Box::new(RuleExpression::Comparison {
                    left: FieldPath::parse("Task.status"),
                    op: CompOp::Eq,
                    right: RuleOperand::String("shipped".into()),
                }),
                then: Box::new(RuleExpression::Comparison {
                    left: FieldPath::parse("Task.shipped_at"),
                    op: CompOp::Neq,
                    right: RuleOperand::Null,
                }),
            },
            modifier: RuleModifier::Strict,
        };

        // Condition met, consequent satisfied
        let row = make_row(vec![
            ("status", Value::String("shipped".into())),
            ("shipped_at", Value::DateTime("2025-01-15T10:30:00".into())),
        ]);
        let all_data = IndexMap::new();
        assert!(evaluate_rule(&rule, "Task", &row, &all_data));

        // Condition met, consequent violated
        let row2 = make_row(vec![
            ("status", Value::String("shipped".into())),
            ("shipped_at", Value::Null),
        ]);
        assert!(!evaluate_rule(&rule, "Task", &row2, &all_data));

        // Condition not met — vacuously true
        let row3 = make_row(vec![
            ("status", Value::String("pending".into())),
            ("shipped_at", Value::Null),
        ]);
        assert!(evaluate_rule(&rule, "Task", &row3, &all_data));
    }

    #[test]
    fn test_probability_modifier() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.priority"),
                op: CompOp::Gt,
                right: RuleOperand::Int(0),
            },
            modifier: RuleModifier::Probability(0.0), // 0% chance of enforcement
        };

        // Rule is violated but probability is 0, so it should pass
        let row = make_row(vec![("priority", Value::Int(-1))]);
        let all_data = IndexMap::new();
        let mut rng = rand::thread_rng();

        let result = enforce_rules(&[rule], "Task", &row, &all_data, &mut rng);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rule_applies_to_entity() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.status"),
                op: CompOp::Eq,
                right: RuleOperand::String("done".into()),
            },
            modifier: RuleModifier::Strict,
        };

        assert!(rule_applies_to_entity(&rule, "Task"));
        assert!(!rule_applies_to_entity(&rule, "Order"));
    }

    #[test]
    fn test_enforce_strict_violation() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.status"),
                op: CompOp::Eq,
                right: RuleOperand::String("done".into()),
            },
            modifier: RuleModifier::Strict,
        };

        let row = make_row(vec![("status", Value::String("pending".into()))]);
        let all_data = IndexMap::new();
        let mut rng = rand::thread_rng();

        let result = enforce_rules(&[rule], "Task", &row, &all_data, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforce_warn_does_not_error() {
        let rule = Rule {
            expression: RuleExpression::Comparison {
                left: FieldPath::parse("Task.status"),
                op: CompOp::Eq,
                right: RuleOperand::String("done".into()),
            },
            modifier: RuleModifier::Warn,
        };

        let row = make_row(vec![("status", Value::String("pending".into()))]);
        let all_data = IndexMap::new();
        let mut rng = rand::thread_rng();

        let result = enforce_rules(&[rule], "Task", &row, &all_data, &mut rng);
        assert!(result.is_ok());
    }
}
