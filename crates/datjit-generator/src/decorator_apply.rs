use datjit_core::error::DatjitError;
use datjit_core::model::decorator::{Decorator, LiteralValue, RangeValue};
use datjit_core::model::entity::Field;
use datjit_core::value::Value;
use rand::Rng;

/// Apply decorator constraints to a generated value, modifying it as needed.
/// Order: non-range/non-multipleOf decorators → range/min/max → multipleOf → range/min/max again.
/// This ensures multipleOf rounding is re-clamped to the valid range.
pub fn apply_decorators(
    value: Value,
    field: &Field,
    rng: &mut impl Rng,
) -> Result<Value, DatjitError> {
    let mut val = value;
    let has_multiple_of = field
        .decorators
        .iter()
        .any(|d| matches!(d, Decorator::MultipleOf(_)));

    // Pass 1: apply non-range, non-multipleOf decorators
    for dec in &field.decorators {
        if matches!(
            dec,
            Decorator::Min { .. }
                | Decorator::Max { .. }
                | Decorator::Range { .. }
                | Decorator::MultipleOf(_)
        ) {
            continue;
        }
        val = apply_single_decorator(val, dec, rng)?;
    }

    // Pass 2: apply range/min/max
    for dec in &field.decorators {
        if matches!(
            dec,
            Decorator::Min { .. } | Decorator::Max { .. } | Decorator::Range { .. }
        ) {
            val = apply_single_decorator(val, dec, rng)?;
        }
    }

    // Pass 3: apply multipleOf
    for dec in &field.decorators {
        if let Decorator::MultipleOf(step) = dec {
            val = apply_multiple_of(val, *step)?;
        }
    }

    // Pass 4: re-snap to valid multiple within range bounds
    if has_multiple_of {
        let step = field.decorators.iter().find_map(|d| match d {
            Decorator::MultipleOf(s) => Some(*s),
            _ => None,
        });
        if let Some(step) = step {
            for dec in &field.decorators {
                match dec {
                    Decorator::Range {
                        lo,
                        hi,
                        lo_exclusive,
                        hi_exclusive,
                    } => {
                        val = snap_multiple_to_range(
                            val,
                            lo,
                            hi,
                            *lo_exclusive,
                            *hi_exclusive,
                            step,
                        )?;
                    }
                    Decorator::Min {
                        value: min_val,
                        exclusive,
                    } => {
                        val = apply_min(val, min_val, *exclusive)?;
                        // Re-snap up if needed
                        val = snap_up_to_multiple(val, step);
                    }
                    Decorator::Max {
                        value: max_val,
                        exclusive,
                    } => {
                        val = apply_max(val, max_val, *exclusive)?;
                        // Re-snap down if needed
                        val = snap_down_to_multiple(val, step);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(val)
}

fn apply_single_decorator(
    value: Value,
    decorator: &Decorator,
    rng: &mut impl Rng,
) -> Result<Value, DatjitError> {
    match decorator {
        Decorator::Min {
            value: range_val,
            exclusive,
        } => apply_min(value, range_val, *exclusive),
        Decorator::Max {
            value: range_val,
            exclusive,
        } => apply_max(value, range_val, *exclusive),
        Decorator::Range {
            lo,
            hi,
            lo_exclusive,
            hi_exclusive,
        } => apply_range(value, lo, hi, *lo_exclusive, *hi_exclusive),
        Decorator::MultipleOf(step) => apply_multiple_of(value, *step),
        Decorator::Len(lo, hi) => apply_len(value, *lo, *hi, rng),
        Decorator::Values(allowed) => apply_values(allowed, rng),
        Decorator::NotEmpty => apply_not_empty(value, rng),
        Decorator::Default(lit) => apply_default(value, lit),
        _ => Ok(value),
    }
}

fn apply_min(value: Value, min_val: &RangeValue, exclusive: bool) -> Result<Value, DatjitError> {
    match (&value, min_val) {
        (Value::Int(n), RangeValue::Int(min)) => {
            let effective = if exclusive { *min + 1 } else { *min };
            Ok(Value::Int((*n).max(effective)))
        }
        (Value::Float(n), RangeValue::Float(min)) => {
            let effective = if exclusive {
                *min + min.abs().max(1.0) * 1e-9
            } else {
                *min
            };
            Ok(Value::Float(n.max(effective)))
        }
        (Value::Float(n), RangeValue::Int(min)) => {
            let min_f = *min as f64;
            let effective = if exclusive {
                min_f + min_f.abs().max(1.0) * 1e-9
            } else {
                min_f
            };
            Ok(Value::Float(n.max(effective)))
        }
        (Value::Int(n), RangeValue::Float(min)) => {
            let effective = if exclusive {
                *min + min.abs().max(1.0) * 1e-9
            } else {
                *min
            };
            Ok(Value::Float((*n as f64).max(effective)))
        }
        _ => Ok(value),
    }
}

fn apply_max(value: Value, max_val: &RangeValue, exclusive: bool) -> Result<Value, DatjitError> {
    match (&value, max_val) {
        (Value::Int(n), RangeValue::Int(max)) => {
            let effective = if exclusive { *max - 1 } else { *max };
            Ok(Value::Int((*n).min(effective)))
        }
        (Value::Float(n), RangeValue::Float(max)) => {
            let effective = if exclusive {
                *max - max.abs().max(1.0) * 1e-9
            } else {
                *max
            };
            Ok(Value::Float(n.min(effective)))
        }
        (Value::Float(n), RangeValue::Int(max)) => {
            let max_f = *max as f64;
            let effective = if exclusive {
                max_f - max_f.abs().max(1.0) * 1e-9
            } else {
                max_f
            };
            Ok(Value::Float(n.min(effective)))
        }
        (Value::Int(n), RangeValue::Float(max)) => {
            let effective = if exclusive {
                *max - max.abs().max(1.0) * 1e-9
            } else {
                *max
            };
            Ok(Value::Float((*n as f64).min(effective)))
        }
        _ => Ok(value),
    }
}

fn apply_range(
    value: Value,
    lo: &RangeValue,
    hi: &RangeValue,
    lo_exclusive: bool,
    hi_exclusive: bool,
) -> Result<Value, DatjitError> {
    match (&value, lo, hi) {
        (Value::Int(n), RangeValue::Int(lo), RangeValue::Int(hi)) => {
            let effective_lo = if lo_exclusive { *lo + 1 } else { *lo };
            let effective_hi = if hi_exclusive { *hi - 1 } else { *hi };
            Ok(Value::Int((*n).clamp(effective_lo, effective_hi)))
        }
        (Value::Float(n), RangeValue::Int(lo), RangeValue::Int(hi)) => {
            let lo_f = *lo as f64;
            let hi_f = *hi as f64;
            let epsilon = ((hi_f - lo_f).abs() * 1e-9).max(1e-10);
            let effective_lo = if lo_exclusive { lo_f + epsilon } else { lo_f };
            let effective_hi = if hi_exclusive { hi_f - epsilon } else { hi_f };
            Ok(Value::Float(n.clamp(effective_lo, effective_hi)))
        }
        (Value::Float(n), RangeValue::Float(lo), RangeValue::Float(hi)) => {
            let epsilon = ((*hi - *lo).abs() * 1e-9).max(1e-10);
            let effective_lo = if lo_exclusive { *lo + epsilon } else { *lo };
            let effective_hi = if hi_exclusive { *hi - epsilon } else { *hi };
            Ok(Value::Float(n.clamp(effective_lo, effective_hi)))
        }
        _ => Ok(value),
    }
}

fn apply_multiple_of(value: Value, step: f64) -> Result<Value, DatjitError> {
    match value {
        Value::Int(n) => {
            let step_i = step as i64;
            if step_i != 0 {
                Ok(Value::Int(((n as f64 / step).round() as i64) * step_i))
            } else {
                Ok(Value::Int(n))
            }
        }
        Value::Float(n) => {
            if step != 0.0 {
                Ok(Value::Float((n / step).round() * step))
            } else {
                Ok(Value::Float(n))
            }
        }
        _ => Ok(value),
    }
}

/// Snap a value to the nearest multiple of `step` that falls within [lo, hi].
fn snap_multiple_to_range(
    value: Value,
    lo: &RangeValue,
    hi: &RangeValue,
    lo_exclusive: bool,
    hi_exclusive: bool,
    step: f64,
) -> Result<Value, DatjitError> {
    match (&value, lo, hi) {
        (Value::Int(n), RangeValue::Int(lo), RangeValue::Int(hi)) => {
            let step_i = step as i64;
            if step_i == 0 {
                return Ok(value);
            }
            let effective_lo = if lo_exclusive { *lo + 1 } else { *lo };
            let effective_hi = if hi_exclusive { *hi - 1 } else { *hi };
            let mut v = *n;
            if v < effective_lo {
                // Snap up to next multiple >= effective_lo
                v = ((effective_lo + step_i - 1) / step_i) * step_i;
            } else if v > effective_hi {
                // Snap down to previous multiple <= effective_hi
                v = (effective_hi / step_i) * step_i;
            }
            Ok(Value::Int(v.clamp(effective_lo, effective_hi)))
        }
        (Value::Float(n), _, _) => {
            let lo_f = match lo {
                RangeValue::Int(v) => *v as f64,
                RangeValue::Float(v) => *v,
                _ => return Ok(value),
            };
            let hi_f = match hi {
                RangeValue::Int(v) => *v as f64,
                RangeValue::Float(v) => *v,
                _ => return Ok(value),
            };
            let epsilon = ((hi_f - lo_f).abs() * 1e-9).max(1e-10);
            let effective_lo = if lo_exclusive { lo_f + epsilon } else { lo_f };
            let effective_hi = if hi_exclusive { hi_f - epsilon } else { hi_f };
            let mut v = *n;
            if v < effective_lo {
                v = (effective_lo / step).ceil() * step;
            } else if v > effective_hi {
                v = (effective_hi / step).floor() * step;
            }
            Ok(Value::Float(v.clamp(effective_lo, effective_hi)))
        }
        _ => Ok(value),
    }
}

fn snap_up_to_multiple(value: Value, step: f64) -> Value {
    match value {
        Value::Int(n) => {
            let step_i = step as i64;
            if step_i != 0 && n % step_i != 0 {
                Value::Int(((n / step_i) + 1) * step_i)
            } else {
                Value::Int(n)
            }
        }
        Value::Float(n) => {
            if step != 0.0 {
                let rounded = (n / step).ceil() * step;
                Value::Float(rounded)
            } else {
                Value::Float(n)
            }
        }
        v => v,
    }
}

fn snap_down_to_multiple(value: Value, step: f64) -> Value {
    match value {
        Value::Int(n) => {
            let step_i = step as i64;
            if step_i != 0 && n % step_i != 0 {
                Value::Int((n / step_i) * step_i)
            } else {
                Value::Int(n)
            }
        }
        Value::Float(n) => {
            if step != 0.0 {
                let rounded = (n / step).floor() * step;
                Value::Float(rounded)
            } else {
                Value::Float(n)
            }
        }
        v => v,
    }
}

fn apply_len(value: Value, lo: usize, hi: usize, rng: &mut impl Rng) -> Result<Value, DatjitError> {
    match value {
        Value::String(s) => {
            let len = s.len();
            if len < lo {
                // Pad with repeated characters
                let mut padded = s;
                while padded.len() < lo {
                    padded.push('x');
                }
                Ok(Value::String(padded))
            } else if len > hi {
                // Truncate
                Ok(Value::String(s[..hi].to_string()))
            } else {
                Ok(Value::String(s))
            }
        }
        Value::List(items) => {
            let len = items.len();
            if len > hi {
                Ok(Value::List(items[..hi].to_vec()))
            } else if len < lo {
                // Pad with Null values
                let mut padded = items;
                while padded.len() < lo {
                    padded.push(Value::Null);
                }
                Ok(Value::List(padded))
            } else {
                Ok(Value::List(items))
            }
        }
        _ => {
            let _ = rng;
            Ok(value)
        }
    }
}

fn apply_values(allowed: &[String], rng: &mut impl Rng) -> Result<Value, DatjitError> {
    if allowed.is_empty() {
        return Err(DatjitError::Generation(
            "@values decorator has empty value list".into(),
        ));
    }
    let idx = rng.gen_range(0..allowed.len());
    Ok(Value::String(allowed[idx].clone()))
}

fn apply_not_empty(value: Value, rng: &mut impl Rng) -> Result<Value, DatjitError> {
    match &value {
        Value::String(s) if s.is_empty() => {
            // Generate a simple placeholder
            let len = rng.gen_range(3..10);
            let placeholder: String = (0..len)
                .map(|_| (b'a' + rng.gen_range(0..26)) as char)
                .collect();
            Ok(Value::String(placeholder))
        }
        Value::List(items) if items.is_empty() => Ok(Value::List(vec![Value::Null])),
        Value::Null => {
            // Treat null as empty
            let len = rng.gen_range(3..10);
            let placeholder: String = (0..len)
                .map(|_| (b'a' + rng.gen_range(0..26)) as char)
                .collect();
            Ok(Value::String(placeholder))
        }
        _ => Ok(value),
    }
}

fn apply_default(value: Value, default_val: &LiteralValue) -> Result<Value, DatjitError> {
    if value.is_null() {
        Ok(literal_to_value(default_val))
    } else {
        Ok(value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::model::entity::Field;
    use datjit_core::types::{PrimitiveType, TypeExpr};

    fn make_field(name: &str, decorators: Vec<Decorator>) -> Field {
        Field::new(name, TypeExpr::Primitive(PrimitiveType::Int(None))).with_decorators(decorators)
    }

    #[test]
    fn test_min_clamp() {
        let field = make_field(
            "age",
            vec![Decorator::Min {
                value: RangeValue::Int(0),
                exclusive: false,
            }],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(-5), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(0));

        let result = apply_decorators(Value::Int(10), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_min_exclusive() {
        let field = make_field(
            "age",
            vec![Decorator::Min {
                value: RangeValue::Int(0),
                exclusive: true,
            }],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(0), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_max_clamp() {
        let field = make_field(
            "score",
            vec![Decorator::Max {
                value: RangeValue::Int(100),
                exclusive: false,
            }],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(150), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_max_exclusive() {
        let field = make_field(
            "score",
            vec![Decorator::Max {
                value: RangeValue::Int(100),
                exclusive: true,
            }],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(100), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_len_string_truncate() {
        let field = make_field("code", vec![Decorator::Len(2, 5)]);
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::String("toolong".into()), &field, &mut rng).unwrap();
        assert_eq!(result, Value::String("toolo".into()));
    }

    #[test]
    fn test_len_string_pad() {
        let field = make_field("code", vec![Decorator::Len(5, 10)]);
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::String("ab".into()), &field, &mut rng).unwrap();
        if let Value::String(s) = &result {
            assert!(s.len() >= 5);
        } else {
            panic!("expected String");
        }
    }

    #[test]
    fn test_values_decorator() {
        let field = make_field(
            "status",
            vec![Decorator::Values(vec!["active".into(), "inactive".into()])],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Null, &field, &mut rng).unwrap();
        if let Value::String(s) = &result {
            assert!(s == "active" || s == "inactive");
        } else {
            panic!("expected String");
        }
    }

    #[test]
    fn test_not_empty() {
        let field = make_field("name", vec![Decorator::NotEmpty]);
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::String("".into()), &field, &mut rng).unwrap();
        if let Value::String(s) = &result {
            assert!(!s.is_empty());
        } else {
            panic!("expected non-empty String");
        }
    }

    #[test]
    fn test_default_on_null() {
        let field = make_field(
            "status",
            vec![Decorator::Default(LiteralValue::String("pending".into()))],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Null, &field, &mut rng).unwrap();
        assert_eq!(result, Value::String("pending".into()));
    }

    #[test]
    fn test_default_not_applied_on_non_null() {
        let field = make_field(
            "status",
            vec![Decorator::Default(LiteralValue::String("pending".into()))],
        );
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::String("active".into()), &field, &mut rng).unwrap();
        assert_eq!(result, Value::String("active".into()));
    }
}
