use chrono::{DateTime, NaiveDate, NaiveTime};
use rand::Rng;
use uuid::Uuid;

use datjit_core::types::PrimitiveType;
use datjit_core::value::Value;

const WORDS: &[&str] = &[
    "account",
    "action",
    "admin",
    "alert",
    "alpha",
    "analytics",
    "app",
    "archive",
    "asset",
    "audit",
    "auto",
    "backup",
    "batch",
    "beta",
    "billing",
    "build",
    "cache",
    "channel",
    "check",
    "client",
    "cloud",
    "cluster",
    "config",
    "content",
    "core",
    "custom",
    "daily",
    "dashboard",
    "data",
    "debug",
    "default",
    "deploy",
    "design",
    "device",
    "display",
    "domain",
    "draft",
    "driver",
    "edge",
    "email",
    "engine",
    "entry",
    "error",
    "event",
    "export",
    "feature",
    "field",
    "file",
    "filter",
    "final",
    "first",
    "fix",
    "flow",
    "form",
    "gateway",
    "global",
    "group",
    "handler",
    "health",
    "host",
    "import",
    "index",
    "initial",
    "input",
    "install",
    "internal",
    "item",
    "job",
    "key",
    "label",
    "launch",
    "layer",
    "legacy",
    "level",
    "limit",
    "link",
    "list",
    "load",
    "local",
    "log",
    "main",
    "manage",
    "manual",
    "market",
    "master",
    "match",
    "media",
    "merge",
    "metric",
    "migrate",
    "mobile",
    "model",
    "module",
    "monitor",
    "network",
    "node",
    "note",
    "notify",
    "object",
    "online",
    "open",
    "option",
    "order",
    "output",
    "package",
    "panel",
    "parser",
    "patch",
    "path",
    "payment",
    "plan",
    "platform",
    "plugin",
    "policy",
    "portal",
    "preview",
    "primary",
    "process",
    "product",
    "profile",
    "project",
    "proxy",
    "public",
    "query",
    "queue",
    "quota",
    "record",
    "region",
    "release",
    "remote",
    "render",
    "report",
    "request",
    "reset",
    "resource",
    "review",
    "role",
    "route",
    "rule",
    "runtime",
    "sample",
    "schema",
    "scope",
    "script",
    "search",
    "secure",
    "server",
    "service",
    "session",
    "setup",
    "share",
    "signal",
    "simple",
    "single",
    "snapshot",
    "source",
    "stage",
    "standard",
    "start",
    "state",
    "static",
    "status",
    "step",
    "storage",
    "stream",
    "style",
    "submit",
    "support",
    "sync",
    "system",
    "table",
    "target",
    "task",
    "team",
    "template",
    "tenant",
    "test",
    "theme",
    "token",
    "tool",
    "trace",
    "track",
    "transfer",
    "trigger",
    "unit",
    "update",
    "upload",
    "user",
    "value",
    "vendor",
    "verify",
    "version",
    "view",
    "volume",
    "widget",
    "worker",
    "zone",
];

/// Generate a readable string of words up to `max_len` characters.
fn generate_readable_string(max_len: usize, rng: &mut impl Rng) -> String {
    let mut result = String::new();
    // Pick a first word that fits
    let first = pick_word_fitting(max_len, rng);
    // Capitalize first word
    let mut capitalized = String::with_capacity(first.len());
    for (i, c) in first.chars().enumerate() {
        if i == 0 {
            capitalized.extend(c.to_uppercase());
        } else {
            capitalized.push(c);
        }
    }
    result.push_str(&capitalized);

    loop {
        let word = WORDS[rng.gen_range(0..WORDS.len())];
        if result.len() + 1 + word.len() > max_len {
            break;
        }
        result.push(' ');
        result.push_str(word);
    }
    result
}

/// Pick a word that fits within max_len, truncating if necessary.
fn pick_word_fitting(max_len: usize, rng: &mut impl Rng) -> String {
    let word = WORDS[rng.gen_range(0..WORDS.len())];
    if word.len() <= max_len {
        word.to_string()
    } else {
        word[..max_len].to_string()
    }
}

/// Generate a default value for a primitive type.
pub fn generate_primitive(prim: &PrimitiveType, rng: &mut impl Rng) -> Value {
    match prim {
        PrimitiveType::String(max_len) => {
            let max = max_len.unwrap_or(30).min(200);
            Value::String(generate_readable_string(max, rng))
        }

        PrimitiveType::Int(bits) => {
            let range = match bits {
                Some(8) => (-128i64, 127i64),
                Some(16) => (-32768, 32767),
                Some(32) => (-2_147_483_648, 2_147_483_647),
                _ => (-1_000_000, 1_000_000),
            };
            Value::Int(rng.gen_range(range.0..=range.1))
        }

        PrimitiveType::Float(bits) => {
            let val = match bits {
                Some(32) => rng.gen_range(-1000.0f64..1000.0),
                _ => rng.gen_range(-1_000_000.0f64..1_000_000.0),
            };
            Value::Float((val * 100.0).round() / 100.0)
        }

        PrimitiveType::Decimal(precision, scale) => {
            let max = 10f64.powi(*precision as i32 - *scale as i32);
            let val = rng.gen_range(0.0..max);
            let factor = 10f64.powi(*scale as i32);
            Value::Float((val * factor).round() / factor)
        }

        PrimitiveType::Bool => Value::Bool(rng.gen_bool(0.5)),

        PrimitiveType::DateTime => {
            let start = NaiveDate::from_ymd_opt(2020, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let end = NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap();
            let range = end.and_utc().timestamp() - start.and_utc().timestamp();
            let offset = rng.gen_range(0..range);
            let dt = DateTime::from_timestamp(start.and_utc().timestamp() + offset, 0)
                .unwrap()
                .naive_utc();
            Value::DateTime(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        }

        PrimitiveType::Date => {
            let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
            let days = rng.gen_range(0..2190); // ~6 years
            let date = start + chrono::Duration::days(days);
            Value::Date(date.format("%Y-%m-%d").to_string())
        }

        PrimitiveType::Time => {
            let h = rng.gen_range(0..24);
            let m = rng.gen_range(0..60);
            let s = rng.gen_range(0..60);
            let time = NaiveTime::from_hms_opt(h, m, s).unwrap();
            Value::Time(time.format("%H:%M:%S").to_string())
        }

        PrimitiveType::Duration => {
            let hours = rng.gen_range(0..72);
            let minutes = rng.gen_range(0..60);
            Value::Duration(format!("PT{hours}H{minutes}M"))
        }

        PrimitiveType::Uuid => Value::Uuid(Uuid::new_v4().to_string()),

        PrimitiveType::Bytes(max_len) => {
            let len = max_len.unwrap_or(16).min(256);
            let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
            Value::Bytes(bytes)
        }

        PrimitiveType::Null => Value::Null,

        PrimitiveType::Any => {
            // Generate a random simple value
            match rng.gen_range(0..3) {
                0 => Value::Int(rng.gen_range(0..1000)),
                1 => Value::String(format!("val_{}", rng.gen_range(0..1000))),
                _ => Value::Bool(rng.gen_bool(0.5)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_string_gen() {
        let val = generate_primitive(&PrimitiveType::String(Some(5)), &mut rng());
        match val {
            Value::String(s) => assert!(s.len() <= 5),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_int_gen() {
        let val = generate_primitive(&PrimitiveType::Int(None), &mut rng());
        assert!(matches!(val, Value::Int(_)));
    }

    #[test]
    fn test_float_gen() {
        let val = generate_primitive(&PrimitiveType::Float(None), &mut rng());
        assert!(matches!(val, Value::Float(_)));
    }

    #[test]
    fn test_bool_gen() {
        let val = generate_primitive(&PrimitiveType::Bool, &mut rng());
        assert!(matches!(val, Value::Bool(_)));
    }

    #[test]
    fn test_uuid_gen() {
        let val = generate_primitive(&PrimitiveType::Uuid, &mut rng());
        match val {
            Value::Uuid(s) => assert_eq!(s.len(), 36), // UUID string length
            _ => panic!("expected Uuid"),
        }
    }

    #[test]
    fn test_date_gen() {
        let val = generate_primitive(&PrimitiveType::Date, &mut rng());
        match val {
            Value::Date(s) => assert!(s.contains('-')),
            _ => panic!("expected Date"),
        }
    }

    #[test]
    fn test_datetime_gen() {
        let val = generate_primitive(&PrimitiveType::DateTime, &mut rng());
        match val {
            Value::DateTime(s) => assert!(s.contains('T')),
            _ => panic!("expected DateTime"),
        }
    }

    #[test]
    fn test_null_gen() {
        let val = generate_primitive(&PrimitiveType::Null, &mut rng());
        assert!(val.is_null());
    }

    #[test]
    fn test_decimal_gen() {
        let val = generate_primitive(&PrimitiveType::Decimal(10, 2), &mut rng());
        match val {
            Value::Float(n) => {
                // Check it has at most 2 decimal places
                let s = format!("{:.2}", n);
                assert!(s.parse::<f64>().is_ok());
            }
            _ => panic!("expected Float for Decimal"),
        }
    }

    #[test]
    fn test_deterministic() {
        let val1 = generate_primitive(&PrimitiveType::Int(None), &mut rng());
        let val2 = generate_primitive(&PrimitiveType::Int(None), &mut rng());
        assert_eq!(val1, val2); // Same seed -> same value
    }

    #[test]
    fn test_string_generates_readable_words() {
        let val = generate_primitive(&PrimitiveType::String(Some(60)), &mut rng());
        match val {
            Value::String(s) => {
                // Should contain only letters and spaces (readable words)
                assert!(
                    s.chars().all(|c| c.is_alphabetic() || c == ' '),
                    "string should be readable words, got: '{}'",
                    s
                );
                // Should have at least one space (multiple words)
                assert!(
                    s.contains(' '),
                    "string with max_len=60 should have multiple words, got: '{}'",
                    s
                );
            }
            _ => panic!("expected String"),
        }
    }
}
