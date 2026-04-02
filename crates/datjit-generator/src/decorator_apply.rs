use datjit_core::error::DatjitError;
use datjit_core::model::decorator::{Decorator, LiteralValue, RangeValue};
use datjit_core::model::entity::Field;
use datjit_core::value::Value;
use rand::Rng;

/// Apply decorator constraints to a generated value, modifying it as needed.
pub fn apply_decorators(
    value: Value,
    field: &Field,
    rng: &mut impl Rng,
) -> Result<Value, DatjitError> {
    let mut val = value;

    for dec in &field.decorators {
        val = apply_single_decorator(val, dec, rng)?;
    }

    Ok(val)
}

fn apply_single_decorator(
    value: Value,
    decorator: &Decorator,
    rng: &mut impl Rng,
) -> Result<Value, DatjitError> {
    match decorator {
        Decorator::Min(range_val) => apply_min(value, range_val),
        Decorator::Max(range_val) => apply_max(value, range_val),
        Decorator::Len(lo, hi) => apply_len(value, *lo, *hi, rng),
        Decorator::Values(allowed) => apply_values(allowed, rng),
        Decorator::NotEmpty => apply_not_empty(value, rng),
        Decorator::Default(lit) => apply_default(value, lit),
        _ => Ok(value),
    }
}

fn apply_min(value: Value, min_val: &RangeValue) -> Result<Value, DatjitError> {
    match (&value, min_val) {
        (Value::Int(n), RangeValue::Int(min)) => {
            Ok(Value::Int((*n).max(*min)))
        }
        (Value::Float(n), RangeValue::Float(min)) => {
            Ok(Value::Float(n.max(*min)))
        }
        (Value::Float(n), RangeValue::Int(min)) => {
            Ok(Value::Float(n.max(*min as f64)))
        }
        (Value::Int(n), RangeValue::Float(min)) => {
            Ok(Value::Float((*n as f64).max(*min)))
        }
        _ => Ok(value),
    }
}

fn apply_max(value: Value, max_val: &RangeValue) -> Result<Value, DatjitError> {
    match (&value, max_val) {
        (Value::Int(n), RangeValue::Int(max)) => {
            Ok(Value::Int((*n).min(*max)))
        }
        (Value::Float(n), RangeValue::Float(max)) => {
            Ok(Value::Float(n.min(*max)))
        }
        (Value::Float(n), RangeValue::Int(max)) => {
            Ok(Value::Float(n.min(*max as f64)))
        }
        (Value::Int(n), RangeValue::Float(max)) => {
            Ok(Value::Float((*n as f64).min(*max)))
        }
        _ => Ok(value),
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
        Value::List(items) if items.is_empty() => {
            Ok(Value::List(vec![Value::Null]))
        }
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
        let field = make_field("age", vec![Decorator::Min(RangeValue::Int(0))]);
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(-5), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(0));

        let result = apply_decorators(Value::Int(10), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_max_clamp() {
        let field = make_field("score", vec![Decorator::Max(RangeValue::Int(100))]);
        let mut rng = rand::thread_rng();
        let result = apply_decorators(Value::Int(150), &field, &mut rng).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_len_string_truncate() {
        let field = make_field("code", vec![Decorator::Len(2, 5)]);
        let mut rng = rand::thread_rng();
        let result =
            apply_decorators(Value::String("toolong".into()), &field, &mut rng).unwrap();
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
            vec![Decorator::Values(vec![
                "active".into(),
                "inactive".into(),
            ])],
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
        let result =
            apply_decorators(Value::String("".into()), &field, &mut rng).unwrap();
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
        let result =
            apply_decorators(Value::String("active".into()), &field, &mut rng).unwrap();
        assert_eq!(result, Value::String("active".into()));
    }
}
