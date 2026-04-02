use rand::Rng;
use uuid::Uuid;

const WORD_LIST: &[&str] = &[
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra", "tango",
    "uniform", "victor", "whiskey", "xray", "yankee", "zulu",
];

/// Expand a pattern template, replacing placeholders like `{AA}`, `{0000}`, `{uuid}`, etc.
///
/// `seq_counter` is incremented each time `{seq}` is encountered.
pub fn expand_pattern(template: &str, rng: &mut impl Rng, seq_counter: &mut u64) -> String {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 16);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            // Find the closing brace
            if let Some(close) = memchr_brace(bytes, i + 1) {
                let placeholder = &template[i + 1..close];
                expand_placeholder(placeholder, rng, seq_counter, &mut result);
                i = close + 1;
            } else {
                // No closing brace, emit literal
                result.push('{');
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Find index of next `}` starting at `start`.
fn memchr_brace(bytes: &[u8], start: usize) -> Option<usize> {
    for i in start..bytes.len() {
        if bytes[i] == b'}' {
            return Some(i);
        }
    }
    None
}

fn expand_placeholder(ph: &str, rng: &mut impl Rng, seq_counter: &mut u64, out: &mut String) {
    match ph {
        "uuid" => {
            out.push_str(&Uuid::new_v4().to_string());
        }
        "seq" => {
            *seq_counter += 1;
            out.push_str(&seq_counter.to_string());
        }
        "word" => {
            let w = WORD_LIST[rng.gen_range(0..WORD_LIST.len())];
            out.push_str(w);
        }
        "WORD" => {
            let w = WORD_LIST[rng.gen_range(0..WORD_LIST.len())];
            out.push_str(&w.to_uppercase());
        }
        _ => {
            // Check pattern structure
            if ph.bytes().all(|b| b == b'A') {
                // Uppercase letters
                for _ in 0..ph.len() {
                    out.push((b'A' + rng.gen_range(0u8..26)) as char);
                }
            } else if ph.bytes().all(|b| b == b'a') {
                // Lowercase letters
                for _ in 0..ph.len() {
                    out.push((b'a' + rng.gen_range(0u8..26)) as char);
                }
            } else if ph.bytes().all(|b| b == b'0') {
                // Zero-padded digits
                let count = ph.len();
                let max = 10u64.pow(count as u32);
                let val = rng.gen_range(0..max);
                let formatted = format!("{:0>width$}", val, width = count);
                out.push_str(&formatted);
            } else if ph.bytes().all(|b| b == b'#') {
                // Hex digits (uppercase)
                for _ in 0..ph.len() {
                    let h = rng.gen_range(0u8..16);
                    out.push_str(&format!("{:X}", h));
                }
            } else {
                // Unknown placeholder, emit as-is
                out.push('{');
                out.push_str(ph);
                out.push('}');
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
    fn test_single_uppercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{A}", &mut r, &mut seq);
        assert_eq!(result.len(), 1);
        assert!(result.chars().next().unwrap().is_ascii_uppercase());
    }

    #[test]
    fn test_multiple_uppercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{AAA}", &mut r, &mut seq);
        assert_eq!(result.len(), 3);
        assert!(result.chars().all(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_single_lowercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{a}", &mut r, &mut seq);
        assert_eq!(result.len(), 1);
        assert!(result.chars().next().unwrap().is_ascii_lowercase());
    }

    #[test]
    fn test_multiple_lowercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{aaa}", &mut r, &mut seq);
        assert_eq!(result.len(), 3);
        assert!(result.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_single_digit() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{0}", &mut r, &mut seq);
        assert_eq!(result.len(), 1);
        assert!(result.chars().next().unwrap().is_ascii_digit());
    }

    #[test]
    fn test_zero_padded_digits() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{0000}", &mut r, &mut seq);
        assert_eq!(result.len(), 4);
        assert!(result.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_hex_digits() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{####}", &mut r, &mut seq);
        assert_eq!(result.len(), 4);
        assert!(result
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));
    }

    #[test]
    fn test_word_lowercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{word}", &mut r, &mut seq);
        assert!(WORD_LIST.contains(&result.as_str()));
    }

    #[test]
    fn test_word_uppercase() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{WORD}", &mut r, &mut seq);
        let lower = result.to_lowercase();
        assert!(WORD_LIST.contains(&lower.as_str()));
    }

    #[test]
    fn test_uuid() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{uuid}", &mut r, &mut seq);
        assert_eq!(result.len(), 36);
        assert!(result.contains('-'));
    }

    #[test]
    fn test_seq_increments() {
        let mut r = rng();
        let mut seq = 0u64;
        let r1 = expand_pattern("{seq}", &mut r, &mut seq);
        let r2 = expand_pattern("{seq}", &mut r, &mut seq);
        let r3 = expand_pattern("{seq}", &mut r, &mut seq);
        assert_eq!(r1, "1");
        assert_eq!(r2, "2");
        assert_eq!(r3, "3");
    }

    #[test]
    fn test_combined_pattern() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("SKU-{AA}-{0000}", &mut r, &mut seq);
        assert_eq!(result.len(), 11); // "SKU-" (4) + 2 letters + "-" (1) + 4 digits
        assert!(result.starts_with("SKU-"));
        let parts: Vec<&str> = result.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_uppercase()));
        assert!(parts[2].len() == 4 && parts[2].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_no_placeholders() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("plain-text", &mut r, &mut seq);
        assert_eq!(result, "plain-text");
    }

    #[test]
    fn test_unknown_placeholder_passthrough() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("{unknown}", &mut r, &mut seq);
        assert_eq!(result, "{unknown}");
    }

    #[test]
    fn test_unclosed_brace() {
        let mut r = rng();
        let mut seq = 0u64;
        let result = expand_pattern("test{AA", &mut r, &mut seq);
        assert_eq!(result, "test{AA");
    }
}
