use datjit_core::error::DatjitError;
use datjit_core::model::decorator::*;

/// Split a field definition string into the type portion and decorators.
/// Example: `"currency.usd @range(1..5000) @dist(lognormal)"` -> `("currency.usd", ["@range(1..5000)", "@dist(lognormal)"])`
pub fn split_type_and_decorators(input: &str) -> (String, Vec<String>) {
    let input = input.trim();
    let mut type_part = String::new();
    let mut decorators = Vec::new();
    let mut current_decorator = String::new();
    let mut in_decorator = false;
    let mut depth = 0i32;

    for ch in input.chars() {
        match ch {
            '@' if depth == 0 => {
                if in_decorator && !current_decorator.is_empty() {
                    decorators.push(current_decorator.trim().to_string());
                }
                current_decorator = String::from("@");
                in_decorator = true;
            }
            '(' if in_decorator => {
                depth += 1;
                current_decorator.push(ch);
            }
            ')' if in_decorator && depth > 0 => {
                depth -= 1;
                current_decorator.push(ch);
            }
            ' ' if in_decorator && depth == 0 && current_decorator.len() > 1 => {
                decorators.push(current_decorator.trim().to_string());
                current_decorator.clear();
                in_decorator = false;
            }
            _ if in_decorator => {
                current_decorator.push(ch);
            }
            _ => {
                type_part.push(ch);
            }
        }
    }

    if in_decorator && !current_decorator.is_empty() {
        decorators.push(current_decorator.trim().to_string());
    }

    (type_part.trim().to_string(), decorators)
}

/// Parse a single decorator string like `@range(1..5000)` into a Decorator.
pub fn parse_decorator(input: &str) -> Result<Decorator, DatjitError> {
    let input = input.trim();
    if !input.starts_with('@') {
        return Err(DatjitError::parse(
            "decorator",
            format!("expected '@', got: {input}"),
        ));
    }

    let input = &input[1..]; // strip @

    // Simple decorators (no args)
    if !input.contains('(') {
        return match input {
            "auto" => Ok(Decorator::Auto),
            "unique" => Ok(Decorator::Unique),
            "primary" => Ok(Decorator::Primary),
            "index" => Ok(Decorator::Index),
            "immutable" => Ok(Decorator::Immutable),
            "not_empty" => Ok(Decorator::NotEmpty),
            "optional" => Ok(Decorator::Optional),
            "readonly" => Ok(Decorator::Readonly),
            "no_delete" => Ok(Decorator::NoDelete),
            "soft_delete" => Ok(Decorator::SoftDelete),
            "sortable" => Ok(Decorator::Sortable),
            "filterable" => Ok(Decorator::Filterable),
            "searchable" => Ok(Decorator::Searchable),
            "hidden" => Ok(Decorator::Hidden),
            "sensitive" => Ok(Decorator::Sensitive),
            "cascade" => Ok(Decorator::Cascade),
            "restrict" => Ok(Decorator::Restrict),
            "set_null" => Ok(Decorator::SetNull),
            "eager" => Ok(Decorator::Eager),
            "lazy" => Ok(Decorator::Lazy),
            "timestamps" => Ok(Decorator::Timestamps),
            "versioned" => Ok(Decorator::Versioned),
            "strict" => Ok(Decorator::Readonly), // handled in rules, not fields
            other => Err(DatjitError::parse(
                "decorator",
                format!("unknown decorator: @{other}"),
            )),
        };
    }

    // Decorators with arguments
    let paren_start = input.find('(').unwrap();
    let name = &input[..paren_start];
    let args_str = &input[paren_start + 1..input.len() - 1]; // strip parens

    match name {
        "range" => parse_range(args_str),
        "min" => {
            let val = parse_range_value(args_str.trim())?;
            Ok(Decorator::Min(val))
        }
        "max" => {
            let val = parse_range_value(args_str.trim())?;
            Ok(Decorator::Max(val))
        }
        "len" => {
            let (lo, hi) = parse_range_pair(args_str)?;
            let lo: usize = lo.parse().map_err(|_| {
                DatjitError::parse("decorator", format!("invalid len lo: {lo}"))
            })?;
            let hi: usize = hi.parse().map_err(|_| {
                DatjitError::parse("decorator", format!("invalid len hi: {hi}"))
            })?;
            Ok(Decorator::Len(lo, hi))
        }
        "pattern" => {
            let pat = args_str.trim().trim_matches('"').to_string();
            // Detect template vs regex: templates have {} placeholders
            if pat.contains('{') && pat.contains('}') {
                Ok(Decorator::Pattern(PatternKind::Template(pat)))
            } else {
                Ok(Decorator::Pattern(PatternKind::Regex(pat)))
            }
        }
        "values" => {
            let values: Vec<String> = args_str.split(',').map(|s| s.trim().to_string()).collect();
            Ok(Decorator::Values(values))
        }
        "default" => {
            let val = parse_literal_value(args_str.trim());
            Ok(Decorator::Default(val))
        }
        "dist" => parse_distribution(args_str),
        "null_rate" => {
            let rate: f64 = args_str.trim().parse().map_err(|_| {
                DatjitError::parse("decorator", format!("invalid null_rate: {args_str}"))
            })?;
            Ok(Decorator::NullRate(rate))
        }
        "count" => {
            if args_str.contains("..") {
                let (lo, hi) = parse_range_pair(args_str)?;
                let lo: usize = lo.parse().map_err(|_| {
                    DatjitError::parse("decorator", format!("invalid count lo: {lo}"))
                })?;
                let hi: usize = hi.parse().map_err(|_| {
                    DatjitError::parse("decorator", format!("invalid count hi: {hi}"))
                })?;
                Ok(Decorator::Count(CountSpec::Range(lo, hi)))
            } else {
                let n: usize = args_str.trim().parse().map_err(|_| {
                    DatjitError::parse("decorator", format!("invalid count: {args_str}"))
                })?;
                Ok(Decorator::Count(CountSpec::Exact(n)))
            }
        }
        "from" => {
            let fields: Vec<String> = args_str.split(',').map(|s| s.trim().to_string()).collect();
            Ok(Decorator::From(fields))
        }
        "after" => Ok(Decorator::After(FieldPath::parse(args_str.trim()))),
        "before" => Ok(Decorator::Before(FieldPath::parse(args_str.trim()))),
        "within" => {
            let parts: Vec<&str> = args_str.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(DatjitError::parse(
                    "decorator",
                    "@within requires (duration, field)",
                ));
            }
            Ok(Decorator::Within(
                DurationLiteral {
                    value: parts[0].trim().to_string(),
                },
                FieldPath::parse(parts[1].trim()),
            ))
        }
        "correlated" => {
            let parts: Vec<&str> = args_str.split(',').collect();
            if parts.len() < 2 {
                return Err(DatjitError::parse(
                    "decorator",
                    "@correlated requires (field, r=value)",
                ));
            }
            let field = parts[0].trim().to_string();
            let r_str = parts[1].trim();
            let r: f64 = r_str
                .strip_prefix("r=")
                .unwrap_or(r_str)
                .parse()
                .map_err(|_| {
                    DatjitError::parse("decorator", format!("invalid correlation: {r_str}"))
                })?;
            Ok(Decorator::Correlated(field, r))
        }
        "derived" => {
            // For now, store as a simple expression placeholder
            // Full expression parsing will be added in Phase 5
            Ok(Decorator::Derived(Expression::FieldRef(FieldPath::parse(
                args_str.trim(),
            ))))
        }
        "paginated" => {
            let size: usize = args_str.trim().parse().map_err(|_| {
                DatjitError::parse("decorator", format!("invalid page size: {args_str}"))
            })?;
            Ok(Decorator::Paginated(size))
        }
        "cacheable" => {
            let ttl: u64 = args_str.trim().parse().map_err(|_| {
                DatjitError::parse("decorator", format!("invalid ttl: {args_str}"))
            })?;
            Ok(Decorator::Cacheable(ttl))
        }
        "domain" => Ok(Decorator::Domain(args_str.trim().to_string())),
        "locale" => Ok(Decorator::Locale(args_str.trim().to_string())),
        "probability" => {
            // This is a rule modifier, not a field decorator — but we parse it here
            // for convenience and it gets applied during rule parsing
            Err(DatjitError::parse(
                "decorator",
                "@probability is a rule modifier, not a field decorator",
            ))
        }
        other => Err(DatjitError::parse(
            "decorator",
            format!("unknown decorator: @{other}"),
        )),
    }
}

/// Parse decorators from a list of strings.
pub fn parse_decorators(inputs: &[String]) -> Result<Vec<Decorator>, DatjitError> {
    inputs.iter().map(|s| parse_decorator(s)).collect()
}

fn parse_range(args: &str) -> Result<Decorator, DatjitError> {
    let (lo_str, hi_str) = parse_range_pair(args)?;
    let lo = parse_range_value(&lo_str)?;
    let hi = parse_range_value(&hi_str)?;
    Ok(Decorator::Range(lo, hi))
}

fn parse_range_pair(input: &str) -> Result<(String, String), DatjitError> {
    let parts: Vec<&str> = input.splitn(2, "..").collect();
    if parts.len() != 2 {
        return Err(DatjitError::parse(
            "decorator",
            format!("expected range with '..' separator, got: {input}"),
        ));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

fn parse_range_value(s: &str) -> Result<RangeValue, DatjitError> {
    let s = s.trim();
    if s == "now" {
        return Ok(RangeValue::Now);
    }
    if s.starts_with("now") {
        return Ok(RangeValue::Relative(s.to_string()));
    }
    if let Ok(n) = s.parse::<i64>() {
        return Ok(RangeValue::Int(n));
    }
    if let Ok(n) = s.parse::<f64>() {
        return Ok(RangeValue::Float(n));
    }
    // Assume it's a date string
    Ok(RangeValue::Date(s.to_string()))
}

fn parse_literal_value(s: &str) -> LiteralValue {
    if s == "null" {
        return LiteralValue::Null;
    }
    if s == "true" {
        return LiteralValue::Bool(true);
    }
    if s == "false" {
        return LiteralValue::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return LiteralValue::Int(n);
    }
    if let Ok(n) = s.parse::<f64>() {
        return LiteralValue::Float(n);
    }
    // Strip quotes if present
    let s = s.trim_matches('"');
    LiteralValue::String(s.to_string())
}

fn parse_distribution(args: &str) -> Result<Decorator, DatjitError> {
    let args = args.trim();

    // Check if it's just numbers (categorical percentages): @dist(70, 25, 5)
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
    if parts.iter().all(|p| p.parse::<f64>().is_ok()) {
        let probs: Vec<f64> = parts.iter().map(|p| p.parse().unwrap()).collect();
        return Ok(Decorator::Dist(Distribution::Categorical(probs)));
    }

    // Named distribution
    let first = parts[0];
    let params: Vec<(&str, &str)> = parts[1..]
        .iter()
        .filter_map(|p| {
            let kv: Vec<&str> = p.splitn(2, '=').collect();
            if kv.len() == 2 {
                Some((kv[0].trim(), kv[1].trim()))
            } else {
                None
            }
        })
        .collect();

    fn _get_param(params: &[(&str, &str)], key: &str) -> Option<f64> {
        params
            .iter()
            .find(|(k, _)| *k == key || *k == format!("\u{03bc}").as_str() || *k == format!("\u{03c3}").as_str() || *k == format!("\u{03bb}").as_str())
            .and_then(|(_, v)| v.parse().ok())
    }

    fn find_param(params: &[(&str, &str)], keys: &[&str]) -> Option<f64> {
        for key in keys {
            if let Some((_, v)) = params.iter().find(|(k, _)| k == key) {
                if let Ok(val) = v.parse::<f64>() {
                    return Some(val);
                }
            }
        }
        None
    }

    match first {
        "uniform" => Ok(Decorator::Dist(Distribution::Uniform)),
        "normal" => {
            let mu = find_param(&params, &["μ", "mu", "mean"]).unwrap_or(0.0);
            let sigma = find_param(&params, &["σ", "sigma", "sd", "std"]).unwrap_or(1.0);
            Ok(Decorator::Dist(Distribution::Normal { mu, sigma }))
        }
        "lognormal" => {
            let mu = find_param(&params, &["μ", "mu"]).unwrap_or(0.0);
            let sigma = find_param(&params, &["σ", "sigma"]).unwrap_or(1.0);
            Ok(Decorator::Dist(Distribution::LogNormal { mu, sigma }))
        }
        "exponential" => {
            let lambda = find_param(&params, &["λ", "lambda"]).unwrap_or(1.0);
            Ok(Decorator::Dist(Distribution::Exponential { lambda }))
        }
        "geometric" => {
            let p = find_param(&params, &["p"]).unwrap_or(0.5);
            Ok(Decorator::Dist(Distribution::Geometric { p }))
        }
        "zipf" => {
            let s = find_param(&params, &["s"]).unwrap_or(1.0);
            Ok(Decorator::Dist(Distribution::Zipf { s }))
        }
        "bimodal" => {
            let peaks_str = args
                .split("peaks=")
                .nth(1)
                .unwrap_or("0,1");
            let peak_vals: Vec<f64> = peaks_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            let p1 = peak_vals.first().copied().unwrap_or(0.0);
            let p2 = peak_vals.get(1).copied().unwrap_or(1.0);
            Ok(Decorator::Dist(Distribution::Bimodal { peaks: (p1, p2) }))
        }
        "weighted" => {
            // @dist(weighted, {v1: w, v2: w})
            // Simplified: parse from remaining args
            let weighted: Vec<(String, f64)> = params
                .iter()
                .map(|(k, v)| (k.to_string(), v.parse::<f64>().unwrap_or(1.0)))
                .collect();
            Ok(Decorator::Dist(Distribution::Weighted(weighted)))
        }
        _ => Err(DatjitError::parse(
            "decorator",
            format!("unknown distribution: {first}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_type_and_decorators() {
        let (ty, decs) =
            split_type_and_decorators("currency.usd @range(1..5000) @dist(lognormal)");
        assert_eq!(ty, "currency.usd");
        assert_eq!(decs.len(), 2);
        assert_eq!(decs[0], "@range(1..5000)");
        assert_eq!(decs[1], "@dist(lognormal)");
    }

    #[test]
    fn test_split_no_decorators() {
        let (ty, decs) = split_type_and_decorators("string");
        assert_eq!(ty, "string");
        assert!(decs.is_empty());
    }

    #[test]
    fn test_split_complex() {
        let (ty, decs) =
            split_type_and_decorators("enum(free, pro, enterprise) @dist(70, 25, 5)");
        assert_eq!(ty, "enum(free, pro, enterprise)");
        assert_eq!(decs.len(), 1);
        assert_eq!(decs[0], "@dist(70, 25, 5)");
    }

    #[test]
    fn test_simple_decorators() {
        assert!(matches!(parse_decorator("@auto").unwrap(), Decorator::Auto));
        assert!(matches!(
            parse_decorator("@unique").unwrap(),
            Decorator::Unique
        ));
        assert!(matches!(
            parse_decorator("@primary").unwrap(),
            Decorator::Primary
        ));
        assert!(matches!(
            parse_decorator("@optional").unwrap(),
            Decorator::Optional
        ));
        assert!(matches!(
            parse_decorator("@searchable").unwrap(),
            Decorator::Searchable
        ));
    }

    #[test]
    fn test_range_int() {
        let d = parse_decorator("@range(1..5000)").unwrap();
        match d {
            Decorator::Range(RangeValue::Int(1), RangeValue::Int(5000)) => {}
            other => panic!("expected Range(1, 5000), got: {other:?}"),
        }
    }

    #[test]
    fn test_range_now() {
        // "2020" without dashes is parsed as integer
        let d = parse_decorator("@range(2020..now)").unwrap();
        match d {
            Decorator::Range(RangeValue::Int(2020), RangeValue::Now) => {}
            other => panic!("expected Range(Int(2020), Now), got: {other:?}"),
        }

        // Date with dashes
        let d = parse_decorator("@range(2020-01-01..now)").unwrap();
        match d {
            Decorator::Range(RangeValue::Date(s), RangeValue::Now) => {
                assert_eq!(s, "2020-01-01");
            }
            other => panic!("expected Range(Date, Now), got: {other:?}"),
        }
    }

    #[test]
    fn test_range_relative() {
        let d = parse_decorator("@range(now-90d..now)").unwrap();
        match d {
            Decorator::Range(RangeValue::Relative(s), RangeValue::Now) => {
                assert_eq!(s, "now-90d");
            }
            other => panic!("expected Range(relative, now), got: {other:?}"),
        }
    }

    #[test]
    fn test_len() {
        let d = parse_decorator("@len(3..60)").unwrap();
        assert_eq!(d, Decorator::Len(3, 60));
    }

    #[test]
    fn test_pattern_template() {
        let d = parse_decorator("@pattern(\"{AA}-{0000}\")").unwrap();
        match d {
            Decorator::Pattern(PatternKind::Template(t)) => {
                assert_eq!(t, "{AA}-{0000}");
            }
            other => panic!("expected Template, got: {other:?}"),
        }
    }

    #[test]
    fn test_dist_categorical() {
        let d = parse_decorator("@dist(70, 25, 5)").unwrap();
        match d {
            Decorator::Dist(Distribution::Categorical(probs)) => {
                assert_eq!(probs, vec![70.0, 25.0, 5.0]);
            }
            other => panic!("expected Categorical, got: {other:?}"),
        }
    }

    #[test]
    fn test_dist_normal() {
        let d = parse_decorator("@dist(normal, mu=35, sigma=12)").unwrap();
        match d {
            Decorator::Dist(Distribution::Normal { mu, sigma }) => {
                assert!((mu - 35.0).abs() < f64::EPSILON);
                assert!((sigma - 12.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Normal, got: {other:?}"),
        }
    }

    #[test]
    fn test_dist_lognormal_no_params() {
        let d = parse_decorator("@dist(lognormal)").unwrap();
        match d {
            Decorator::Dist(Distribution::LogNormal { mu, sigma }) => {
                assert!((mu - 0.0).abs() < f64::EPSILON);
                assert!((sigma - 1.0).abs() < f64::EPSILON);
            }
            other => panic!("expected LogNormal, got: {other:?}"),
        }
    }

    #[test]
    fn test_null_rate() {
        let d = parse_decorator("@null_rate(0.3)").unwrap();
        match d {
            Decorator::NullRate(r) => assert!((r - 0.3).abs() < f64::EPSILON),
            other => panic!("expected NullRate, got: {other:?}"),
        }
    }

    #[test]
    fn test_count_range() {
        let d = parse_decorator("@count(0..20)").unwrap();
        assert_eq!(d, Decorator::Count(CountSpec::Range(0, 20)));
    }

    #[test]
    fn test_count_exact() {
        let d = parse_decorator("@count(5)").unwrap();
        assert_eq!(d, Decorator::Count(CountSpec::Exact(5)));
    }

    #[test]
    fn test_default_string() {
        let d = parse_decorator("@default(\"US\")").unwrap();
        assert_eq!(d, Decorator::Default(LiteralValue::String("US".into())));
    }

    #[test]
    fn test_default_int() {
        let d = parse_decorator("@default(0)").unwrap();
        assert_eq!(d, Decorator::Default(LiteralValue::Int(0)));
    }

    #[test]
    fn test_after() {
        let d = parse_decorator("@after(start_date)").unwrap();
        match d {
            Decorator::After(fp) => assert_eq!(fp.segments, vec!["start_date"]),
            other => panic!("expected After, got: {other:?}"),
        }
    }
}
