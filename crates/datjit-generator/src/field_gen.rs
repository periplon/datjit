use indexmap::IndexMap;
use rand::Rng;

use datjit_core::error::DatjitError;
use datjit_core::model::decorator::{Decorator, PatternKind, RangeValue};
use datjit_core::model::entity::Field;
use datjit_core::types::*;
use datjit_core::value::Value;

use crate::context::GenerationContext;
use crate::distribution::sample_distribution;
use crate::pattern::expand_pattern;
use crate::primitive_gen::generate_primitive;

/// Generate a value for a single field.
pub fn generate_field(
    field: &Field,
    entity_name: &str,
    _row_so_far: &IndexMap<String, Value>,
    ctx: &mut GenerationContext,
) -> Result<Value, DatjitError> {
    // Check @null_rate
    for dec in &field.decorators {
        if let Decorator::NullRate(rate) = dec {
            if ctx.rng.gen_bool(*rate) {
                return Ok(Value::Null);
            }
        }
    }

    // Check @optional with no default -> sometimes null
    if field.is_optional() && ctx.rng.gen_bool(0.15) {
        return Ok(Value::Null);
    }

    // Generate based on type
    let value = generate_for_type(&field.type_expr, &field.decorators, entity_name, ctx)?;

    // Apply range constraint
    let value = apply_range_constraint(value, &field.decorators, ctx)?;

    Ok(value)
}

fn generate_for_type(
    type_expr: &TypeExpr,
    decorators: &[Decorator],
    entity_name: &str,
    ctx: &mut GenerationContext,
) -> Result<Value, DatjitError> {
    // Check for @pattern(Template(...)) — applies to any type that would produce a string
    for dec in decorators {
        if let Decorator::Pattern(PatternKind::Template(tmpl)) = dec {
            let counter_key = format!("{entity_name}.__pattern_seq");
            let counter = ctx.counters.entry(counter_key).or_insert(0);
            let result = expand_pattern(tmpl, &mut ctx.rng, counter);
            return Ok(Value::String(result));
        }
    }

    match type_expr {
        TypeExpr::Primitive(prim) => {
            // Check for @dist decorator — use distribution sampling for Int/Float
            for dec in decorators {
                if let Decorator::Dist(dist) = dec {
                    match prim {
                        PrimitiveType::Int(_) | PrimitiveType::Float(_) | PrimitiveType::Decimal(_, _) => {
                            let range = extract_range_f64(decorators);
                            let sampled = sample_distribution(dist, range, &mut ctx.rng);
                            return match prim {
                                PrimitiveType::Int(_) => Ok(Value::Int(sampled.round() as i64)),
                                PrimitiveType::Float(_) => Ok(Value::Float((sampled * 100.0).round() / 100.0)),
                                PrimitiveType::Decimal(_, scale) => {
                                    let factor = 10f64.powi(*scale as i32);
                                    Ok(Value::Float((sampled * factor).round() / factor))
                                }
                                _ => unreachable!(),
                            };
                        }
                        _ => {}
                    }
                }
            }
            Ok(generate_primitive(prim, &mut ctx.rng))
        }

        TypeExpr::Semantic(st) => {
            // Fallback to simple generation based on semantic type
            generate_semantic_fallback(st, &mut ctx.rng)
        }

        TypeExpr::Enum(enum_ref) => {
            generate_enum(enum_ref, decorators, &mut ctx.rng)
        }

        TypeExpr::Reference(ref_type) => {
            generate_reference(ref_type, entity_name, ctx)
        }

        TypeExpr::Compound(compound) => match compound {
            CompoundType::Nullable(inner) => {
                if ctx.rng.gen_bool(0.2) {
                    Ok(Value::Null)
                } else {
                    generate_for_type(inner, decorators, entity_name, ctx)
                }
            }
            CompoundType::List(inner) => {
                let count = ctx.rng.gen_range(0..5);
                let items: Result<Vec<_>, _> = (0..count)
                    .map(|_| generate_for_type(inner, &[], entity_name, ctx))
                    .collect();
                Ok(Value::List(items?))
            }
            CompoundType::Union(types) => {
                let idx = ctx.rng.gen_range(0..types.len());
                generate_for_type(&types[idx], decorators, entity_name, ctx)
            }
            CompoundType::Tuple(types) => {
                let items: Result<Vec<_>, _> = types
                    .iter()
                    .map(|t| generate_for_type(t, &[], entity_name, ctx))
                    .collect();
                Ok(Value::Tuple(items?))
            }
            CompoundType::Map(key_type, val_type) => {
                let count = ctx.rng.gen_range(1..5);
                let pairs: Result<Vec<_>, _> = (0..count)
                    .map(|_| {
                        let k = generate_for_type(key_type, &[], entity_name, ctx)?;
                        let v = generate_for_type(val_type, &[], entity_name, ctx)?;
                        Ok::<_, DatjitError>((k.to_output_string(), v))
                    })
                    .collect();
                Ok(Value::Map(pairs?))
            }
        },

        TypeExpr::Named(_name) => {
            // Named types will be expanded in a later phase
            Ok(generate_primitive(&PrimitiveType::String(None), &mut ctx.rng))
        }
    }
}

/// Extract a numeric range from decorators as (f64, f64), if present.
fn extract_range_f64(decorators: &[Decorator]) -> Option<(f64, f64)> {
    for dec in decorators {
        if let Decorator::Range(lo, hi) = dec {
            let lo_f = match lo {
                RangeValue::Int(n) => *n as f64,
                RangeValue::Float(n) => *n,
                _ => continue,
            };
            let hi_f = match hi {
                RangeValue::Int(n) => *n as f64,
                RangeValue::Float(n) => *n,
                _ => continue,
            };
            return Some((lo_f, hi_f));
        }
    }
    None
}

fn generate_semantic_fallback(st: &SemanticType, rng: &mut impl Rng) -> Result<Value, DatjitError> {
    let full = st.full_name();
    let val = match full.as_str() {
        "person.full" => {
            let firsts = ["James", "Maria", "Chen", "Fatima", "Liam", "Sofia", "Alex", "Priya"];
            let lasts = ["Smith", "Garcia", "Wang", "Patel", "Johnson", "Kim", "Santos", "Mueller"];
            let f = firsts[rng.gen_range(0..firsts.len())];
            let l = lasts[rng.gen_range(0..lasts.len())];
            Value::String(format!("{f} {l}"))
        }
        "person.first" => {
            let names = ["James", "Maria", "Chen", "Fatima", "Liam", "Sofia", "Alex", "Priya", "Omar", "Yuki"];
            Value::String(names[rng.gen_range(0..names.len())].into())
        }
        "person.last" => {
            let names = ["Smith", "Garcia", "Wang", "Patel", "Johnson", "Kim", "Santos", "Mueller", "Nakamura", "Ali"];
            Value::String(names[rng.gen_range(0..names.len())].into())
        }
        "person.username" => {
            Value::String(format!("user{}", rng.gen_range(100..9999)))
        }
        "person.gender" => {
            let genders = ["female", "male", "nonbinary"];
            Value::String(genders[rng.gen_range(0..genders.len())].into())
        }
        "person.prefix" => {
            let prefixes = ["Mr.", "Ms.", "Dr.", "Prof."];
            Value::String(prefixes[rng.gen_range(0..prefixes.len())].into())
        }
        "person.dob" => {
            let year = rng.gen_range(1950..2005);
            let month = rng.gen_range(1..=12);
            let day = rng.gen_range(1..=28);
            Value::Date(format!("{year:04}-{month:02}-{day:02}"))
        }
        "person.age" => Value::Int(rng.gen_range(18..85)),
        "person.bio" => {
            Value::String(format!("Experienced professional with {}+ years in the industry.", rng.gen_range(1..30)))
        }
        "person.avatar" => {
            Value::String(format!("https://i.pravatar.cc/150?u={}", rng.gen_range(1..10000)))
        }
        "email" => {
            let domains = ["example.com", "test.org", "mail.net", "demo.io"];
            let user = format!("user{}", rng.gen_range(100..9999));
            Value::String(format!("{}@{}", user, domains[rng.gen_range(0..domains.len())]))
        }
        "phone" | "phone.mobile" | "phone.landline" => {
            Value::String(format!("+1-555-{:03}-{:04}", rng.gen_range(100..999), rng.gen_range(1000..9999)))
        }
        "url" => {
            Value::String(format!("https://example.com/page/{}", rng.gen_range(1..10000)))
        }
        "url.image" => {
            Value::String(format!("https://picsum.photos/400/300?id={}", rng.gen_range(1..1000)))
        }
        "url.avatar" => {
            Value::String(format!("https://i.pravatar.cc/150?u={}", rng.gen_range(1..10000)))
        }
        "ipv4" => {
            Value::String(format!("{}.{}.{}.{}", rng.gen_range(1..255), rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(1..255)))
        }
        "ipv6" => {
            let parts: Vec<String> = (0..8).map(|_| format!("{:04x}", rng.gen_range(0u16..0xFFFF))).collect();
            Value::String(parts.join(":"))
        }
        "mac" => {
            let parts: Vec<String> = (0..6).map(|_| format!("{:02X}", rng.gen_range(0u8..255))).collect();
            Value::String(parts.join(":"))
        }
        "address.full" => {
            let street_num = rng.gen_range(100..9999);
            let streets = ["Main St", "Oak Ave", "Park Blvd", "Elm Dr", "Cedar Ln"];
            let cities = ["Springfield", "Portland", "Austin", "Denver", "Seattle"];
            let states = ["IL", "OR", "TX", "CO", "WA"];
            let idx = rng.gen_range(0..cities.len());
            Value::String(format!("{} {}, {}, {} {}", street_num, streets[rng.gen_range(0..streets.len())], cities[idx], states[idx], rng.gen_range(10000..99999)))
        }
        "address.street" => {
            let streets = ["Main St", "Oak Ave", "Park Blvd", "Elm Dr", "Cedar Ln", "Maple Way"];
            Value::String(format!("{} {}", rng.gen_range(100..9999), streets[rng.gen_range(0..streets.len())]))
        }
        "address.city" => {
            let cities = ["Springfield", "Portland", "Austin", "Denver", "Seattle", "Chicago", "Miami"];
            Value::String(cities[rng.gen_range(0..cities.len())].into())
        }
        "address.state" => {
            let states = ["IL", "OR", "TX", "CO", "WA", "CA", "NY", "FL"];
            Value::String(states[rng.gen_range(0..states.len())].into())
        }
        "address.zip" => {
            Value::String(format!("{:05}", rng.gen_range(10000..99999)))
        }
        "address.country" => {
            let countries = ["US", "CA", "GB", "DE", "FR", "JP", "AU"];
            Value::String(countries[rng.gen_range(0..countries.len())].into())
        }
        "geo.lat" => Value::Float((rng.gen_range(-90.0f64..90.0) * 10000.0).round() / 10000.0),
        "geo.lng" => Value::Float((rng.gen_range(-180.0f64..180.0) * 10000.0).round() / 10000.0),
        "timezone" => {
            let tzs = ["America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles", "Europe/London", "Asia/Tokyo"];
            Value::String(tzs[rng.gen_range(0..tzs.len())].into())
        }
        "currency.usd" | "currency.eur" => {
            Value::Float((rng.gen_range(1.0f64..1000.0) * 100.0).round() / 100.0)
        }
        "credit_card" => {
            Value::String(format!("4111-{:04}-{:04}-{:04}", rng.gen_range(1000..9999), rng.gen_range(1000..9999), rng.gen_range(1000..9999)))
        }
        "credit_card.type" => {
            let types = ["visa", "mastercard", "amex", "discover"];
            Value::String(types[rng.gen_range(0..types.len())].into())
        }
        "text.word" => {
            let words = ["ephemeral", "quantum", "nexus", "cipher", "vertex", "axiom", "prism"];
            Value::String(words[rng.gen_range(0..words.len())].into())
        }
        "text.sentence" => {
            Value::String("The quick brown fox jumps over the lazy dog.".into())
        }
        "text.paragraph" => {
            Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.".into())
        }
        "text.slug" => {
            let words = ["great", "new", "fast", "smart", "cool", "best"];
            let nouns = ["post", "item", "page", "article", "product"];
            Value::String(format!("{}-{}-{}", words[rng.gen_range(0..words.len())], nouns[rng.gen_range(0..nouns.len())], rng.gen_range(1..999)))
        }
        "product.title" => {
            let adjs = ["Wireless", "Smart", "Pro", "Ultra", "Mini", "Eco"];
            let nouns = ["Speaker", "Headphones", "Charger", "Display", "Keyboard", "Camera"];
            Value::String(format!("{} {}", adjs[rng.gen_range(0..adjs.len())], nouns[rng.gen_range(0..nouns.len())]))
        }
        "product.description" => {
            Value::String("High-quality product designed for everyday use with premium features.".into())
        }
        "product.sku" => {
            Value::String(format!("SKU-{}{}-{:04}", (b'A' + rng.gen_range(0..26)) as char, (b'A' + rng.gen_range(0..26)) as char, rng.gen_range(1..9999)))
        }
        "company.name" => {
            let prefixes = ["Apex", "Meridian", "Nova", "Atlas", "Zenith", "Vertex"];
            let cores = ["Tech", "Systems", "Solutions", "Digital", "Labs"];
            Value::String(format!("{} {} Inc.", prefixes[rng.gen_range(0..prefixes.len())], cores[rng.gen_range(0..cores.len())]))
        }
        "company.industry" => {
            let industries = ["Technology", "Healthcare", "Finance", "Manufacturing", "Retail"];
            Value::String(industries[rng.gen_range(0..industries.len())].into())
        }
        "company.catch_phrase" => {
            Value::String("Innovate. Integrate. Excel.".into())
        }
        "job.title" => {
            let levels = ["Junior", "Senior", "Lead", "Staff", "Principal"];
            let roles = ["Software Engineer", "Product Manager", "Data Scientist", "Designer", "Analyst"];
            Value::String(format!("{} {}", levels[rng.gen_range(0..levels.len())], roles[rng.gen_range(0..roles.len())]))
        }
        "job.department" => {
            let depts = ["Engineering", "Product", "Design", "Marketing", "Sales", "Operations"];
            Value::String(depts[rng.gen_range(0..depts.len())].into())
        }
        "color.hex" => {
            Value::String(format!("#{:06X}", rng.gen_range(0u32..0xFFFFFF)))
        }
        "color.rgb" => {
            Value::String(format!("rgb({}, {}, {})", rng.gen_range(0..256), rng.gen_range(0..256), rng.gen_range(0..256)))
        }
        "color.name" => {
            let colors = ["cerulean", "crimson", "emerald", "amber", "violet", "teal", "coral"];
            Value::String(colors[rng.gen_range(0..colors.len())].into())
        }
        "file.name" => {
            let names = ["report", "document", "image", "data", "backup"];
            let exts = ["pdf", "docx", "png", "csv", "zip"];
            Value::String(format!("{}_{}.{}", names[rng.gen_range(0..names.len())], rng.gen_range(1..999), exts[rng.gen_range(0..exts.len())]))
        }
        "file.extension" => {
            let exts = [".pdf", ".docx", ".png", ".csv", ".zip", ".json"];
            Value::String(exts[rng.gen_range(0..exts.len())].into())
        }
        "file.mime" => {
            let mimes = ["application/pdf", "image/png", "text/csv", "application/json"];
            Value::String(mimes[rng.gen_range(0..mimes.len())].into())
        }
        "sku" => Value::String(format!("SKU-{}{}-{:04}", (b'A' + rng.gen_range(0..26)) as char, (b'A' + rng.gen_range(0..26)) as char, rng.gen_range(1..9999))),
        "slug" => Value::String(format!("item-{}", rng.gen_range(1000..9999))),
        "code" => Value::String(format!("{}{}{}{:03}", (b'A' + rng.gen_range(0..26)) as char, (b'A' + rng.gen_range(0..26)) as char, (b'A' + rng.gen_range(0..26)) as char, rng.gen_range(1..999))),
        "hash.md5" => {
            let hex: String = (0..32).map(|_| format!("{:x}", rng.gen_range(0u8..16))).collect();
            Value::String(hex)
        }
        "hash.sha256" => {
            let hex: String = (0..64).map(|_| format!("{:x}", rng.gen_range(0u8..16))).collect();
            Value::String(hex)
        }
        "iban" => Value::String(format!("DE{:02}{:04}{:04}{:04}{:04}{:02}", rng.gen_range(10..99), rng.gen_range(1000..9999), rng.gen_range(1000..9999), rng.gen_range(1000..9999), rng.gen_range(1000..9999), rng.gen_range(10..99))),
        "swift" => Value::String(format!("{}{}{}XXX", "COBA", "DE", "FF")),
        _ => {
            // Unknown semantic type — fall back to string
            Value::String(format!("{}_{}", st.full_name(), rng.gen_range(1..10000)))
        }
    };
    Ok(val)
}

fn generate_enum(
    enum_ref: &EnumRef,
    decorators: &[Decorator],
    rng: &mut impl Rng,
) -> Result<Value, DatjitError> {
    let values = match enum_ref {
        EnumRef::Inline(vals) => vals.clone(),
        EnumRef::Named(name) => {
            // Named enums should be resolved before generation
            return Err(DatjitError::Generation(format!(
                "unresolved named enum: {name}"
            )));
        }
    };

    if values.is_empty() {
        return Err(DatjitError::Generation("empty enum".into()));
    }

    // Check for @dist categorical
    for dec in decorators {
        if let Decorator::Dist(datjit_core::model::decorator::Distribution::Categorical(probs)) = dec {
            if probs.len() == values.len() {
                let total: f64 = probs.iter().sum();
                let mut roll = rng.gen_range(0.0..total);
                for (i, prob) in probs.iter().enumerate() {
                    roll -= prob;
                    if roll <= 0.0 {
                        return Ok(Value::String(values[i].clone()));
                    }
                }
                return Ok(Value::String(values.last().unwrap().clone()));
            }
        }
    }

    // Uniform selection
    let idx = rng.gen_range(0..values.len());
    Ok(Value::String(values[idx].clone()))
}

fn generate_reference(
    ref_type: &ReferenceType,
    entity_name: &str,
    ctx: &mut GenerationContext,
) -> Result<Value, DatjitError> {
    match ref_type {
        ReferenceType::BelongsTo { target, optional } => {
            let row_count = ctx.entity_rows(target).len();
            if row_count == 0 {
                if *optional {
                    return Ok(Value::Null);
                }
                return Err(DatjitError::Generation(format!(
                    "no rows generated for referenced entity: {target}"
                )));
            }
            let idx = ctx.rng.gen_range(0..row_count);
            let pk_val = ctx.entity_rows(target)[idx]
                .values()
                .next()
                .cloned()
                .unwrap_or(Value::Null);
            Ok(Value::Ref(target.clone(), Box::new(pk_val)))
        }
        ReferenceType::SelfRef { optional } => {
            let row_count = ctx.entity_rows(entity_name).len();
            if row_count == 0 || *optional && ctx.rng.gen_bool(0.7) {
                return Ok(Value::Null);
            }
            let idx = ctx.rng.gen_range(0..row_count);
            let pk_val = ctx.entity_rows(entity_name)[idx]
                .values()
                .next()
                .cloned()
                .unwrap_or(Value::Null);
            Ok(Value::Ref(entity_name.to_string(), Box::new(pk_val)))
        }
        ReferenceType::HasMany { .. } | ReferenceType::ManyToMany { .. } => {
            Ok(Value::List(Vec::new()))
        }
        ReferenceType::Polymorphic { targets } => {
            if targets.is_empty() {
                return Ok(Value::Null);
            }
            let target_idx = ctx.rng.gen_range(0..targets.len());
            let target = targets[target_idx].clone();
            let row_count = ctx.entity_rows(&target).len();
            if row_count == 0 {
                return Ok(Value::Null);
            }
            let idx = ctx.rng.gen_range(0..row_count);
            let pk_val = ctx.entity_rows(&target)[idx]
                .values()
                .next()
                .cloned()
                .unwrap_or(Value::Null);
            Ok(Value::Ref(target, Box::new(pk_val)))
        }
    }
}

fn apply_range_constraint(
    value: Value,
    decorators: &[Decorator],
    _ctx: &mut GenerationContext,
) -> Result<Value, DatjitError> {
    for dec in decorators {
        if let Decorator::Range(lo, hi) = dec {
            match (&value, lo, hi) {
                (Value::Int(n), RangeValue::Int(lo), RangeValue::Int(hi)) => {
                    let clamped = (*n).clamp(*lo, *hi);
                    return Ok(Value::Int(clamped));
                }
                (Value::Float(n), RangeValue::Int(lo), RangeValue::Int(hi)) => {
                    let clamped = n.clamp(*lo as f64, *hi as f64);
                    return Ok(Value::Float(clamped));
                }
                (Value::Float(n), RangeValue::Float(lo), RangeValue::Float(hi)) => {
                    let clamped = n.clamp(*lo, *hi);
                    return Ok(Value::Float(clamped));
                }
                _ => {}
            }
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::types::PrimitiveType;

    #[test]
    fn test_generate_primitive_field() {
        let field = Field::new("name", TypeExpr::Primitive(PrimitiveType::String(None)));
        let mut ctx = GenerationContext::new(Some(42), "en-US".into());
        let val = generate_field(&field, "User", &IndexMap::new(), &mut ctx).unwrap();
        assert!(matches!(val, Value::String(_)));
    }

    #[test]
    fn test_generate_semantic_field() {
        let field = Field::new("name", TypeExpr::Semantic(SemanticType::new("person", "full")));
        let mut ctx = GenerationContext::new(Some(42), "en-US".into());
        let val = generate_field(&field, "User", &IndexMap::new(), &mut ctx).unwrap();
        match val {
            Value::String(s) => assert!(s.contains(' ')), // first + last
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_generate_enum_with_dist() {
        let field = Field::new(
            "tier",
            TypeExpr::Enum(EnumRef::Inline(vec!["free".into(), "pro".into(), "enterprise".into()])),
        ).with_decorators(vec![
            Decorator::Dist(datjit_core::model::decorator::Distribution::Categorical(vec![70.0, 25.0, 5.0])),
        ]);

        let mut ctx = GenerationContext::new(Some(42), "en-US".into());
        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let val = generate_field(&field, "User", &IndexMap::new(), &mut ctx).unwrap();
            if let Value::String(s) = val {
                *counts.entry(s).or_insert(0) += 1;
            }
        }
        // Free should be most common
        assert!(counts.get("free").unwrap_or(&0) > counts.get("enterprise").unwrap_or(&0));
    }
}
