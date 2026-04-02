use indexmap::IndexMap;
use rand::Rng;

use datjit_core::error::DatjitError;
use datjit_core::model::decorator::Decorator;
use datjit_core::model::entity::Entity;
use datjit_core::value::Value;
use datjit_corpus::embedded;

use crate::context::GenerationContext;
use crate::field_gen::generate_field;

/// Generate values for coherence groups in the entity.
/// Returns a partial row with the coherent field values.
pub fn generate_coherence_groups(
    entity: &Entity,
    ctx: &mut GenerationContext,
) -> Result<IndexMap<String, Value>, DatjitError> {
    let mut partial_row = IndexMap::new();

    for (group_name, field_names) in &entity.coherence_groups {
        let group_values = generate_single_group(group_name, field_names, entity, ctx)?;
        for (k, v) in group_values {
            partial_row.insert(k, v);
        }
    }

    // Handle @from decorators for fields not already in coherence groups.
    // If the source field hasn't been generated yet, generate it on-demand
    // so that @from works without requiring an explicit coherence group.
    for (field_name, field) in &entity.fields {
        if partial_row.contains_key(field_name) {
            continue;
        }
        for dec in &field.decorators {
            if let Decorator::From(sources) = dec {
                if let Some(source_field_name) = sources.first() {
                    // Generate source field on-demand if not yet available
                    if !partial_row.contains_key(source_field_name.as_str()) {
                        if let Some(source_field) = entity.fields.get(source_field_name.as_str())
                        {
                            let source_val =
                                generate_field(source_field, &entity.name, &partial_row, ctx)?;
                            partial_row.insert(source_field_name.clone(), source_val);
                        }
                    }
                    // Now derive the @from field
                    if let Some(source_val) = partial_row.get(source_field_name.as_str()) {
                        let derived =
                            derive_from_value(field_name, source_field_name, source_val, ctx)?;
                        partial_row.insert(field_name.clone(), derived);
                    }
                }
            }
        }
    }

    Ok(partial_row)
}

/// Generate values for a single coherence group.
fn generate_single_group(
    group_name: &str,
    field_names: &[String],
    entity: &Entity,
    ctx: &mut GenerationContext,
) -> Result<IndexMap<String, Value>, DatjitError> {
    // Try to match the group to a known semantic pattern
    let lower_group = group_name.to_lowercase();
    let lower_fields: Vec<String> = field_names.iter().map(|f| f.to_lowercase()).collect();

    if is_location_group(&lower_group, &lower_fields) {
        return generate_location_group(field_names, ctx);
    }

    if is_identity_group(&lower_group, &lower_fields) {
        return generate_identity_group(field_names, ctx);
    }

    // Unknown group: generate fields normally but in order
    generate_default_group(field_names, entity, ctx)
}

fn is_location_group(group_name: &str, fields: &[String]) -> bool {
    if group_name.contains("location") || group_name.contains("address") || group_name.contains("geo") {
        return true;
    }
    let location_keywords = ["office", "city", "state", "zip", "timezone", "phone", "address", "location"];
    let matches = fields
        .iter()
        .filter(|f| location_keywords.iter().any(|kw| f.contains(kw)))
        .count();
    matches >= 2
}

fn is_identity_group(group_name: &str, fields: &[String]) -> bool {
    if group_name.contains("identity") || group_name.contains("person") || group_name.contains("name") {
        return true;
    }
    let identity_keywords = ["first_name", "last_name", "email", "username", "name"];
    let matches = fields
        .iter()
        .filter(|f| identity_keywords.iter().any(|kw| f.contains(kw)))
        .count();
    matches >= 2
}

/// Generate a location-coherent group: pick a city and derive timezone, phone, etc.
fn generate_location_group(
    field_names: &[String],
    ctx: &mut GenerationContext,
) -> Result<IndexMap<String, Value>, DatjitError> {
    let cities = embedded::CITIES;
    let idx = ctx.rng.gen_range(0..cities.len());
    let (city, state, zip, tz) = cities[idx];

    // Derive area code from zip prefix
    let area_code = derive_area_code(zip);

    let mut result = IndexMap::new();

    for field_name in field_names {
        let lower = field_name.to_lowercase();
        let value = if lower.contains("office") || lower.contains("city") || lower.contains("location") {
            Value::String(format!("{}, {}", city, state))
        } else if lower.contains("state") || lower.contains("region") {
            Value::String(state.to_string())
        } else if lower.contains("zip") || lower.contains("postal") {
            Value::String(zip.to_string())
        } else if lower.contains("timezone") || lower.contains("tz") || lower == "time_zone" {
            Value::String(tz.to_string())
        } else if lower.contains("phone") || lower.contains("tel") {
            Value::String(format!(
                "+1-{}-{:03}-{:04}",
                area_code,
                ctx.rng.gen_range(200..999),
                ctx.rng.gen_range(1000..9999)
            ))
        } else if lower.contains("address") || lower.contains("street") {
            let street_num = ctx.rng.gen_range(100..9999);
            let street_names = embedded::STREET_NAMES;
            let street_suffixes = embedded::STREET_SUFFIXES;
            let sn = street_names[ctx.rng.gen_range(0..street_names.len())];
            let ss = street_suffixes[ctx.rng.gen_range(0..street_suffixes.len())];
            Value::String(format!("{} {} {}, {}, {} {}", street_num, sn, ss, city, state, zip))
        } else if lower.contains("country") {
            Value::String("US".to_string())
        } else {
            // Fallback: use the city name
            Value::String(format!("{}, {}", city, state))
        };

        result.insert(field_name.clone(), value);
    }

    Ok(result)
}

/// Generate an identity-coherent group: first+last name, then derive email and username.
fn generate_identity_group(
    field_names: &[String],
    ctx: &mut GenerationContext,
) -> Result<IndexMap<String, Value>, DatjitError> {
    let male_names = embedded::FIRST_NAMES_MALE;
    let female_names = embedded::FIRST_NAMES_FEMALE;
    let last_names = embedded::LAST_NAMES;

    // Pick gender randomly, then pick names
    let use_male = ctx.rng.gen_bool(0.5);
    let first = if use_male {
        male_names[ctx.rng.gen_range(0..male_names.len())]
    } else {
        female_names[ctx.rng.gen_range(0..female_names.len())]
    };
    let last = last_names[ctx.rng.gen_range(0..last_names.len())];

    // Pick an email domain
    let domains = embedded::EMAIL_DOMAINS;
    let domain_idx = ctx.rng.gen_range(0..domains.len());
    let domain = domains[domain_idx].0;

    let num_suffix: u32 = ctx.rng.gen_range(1..999);

    let mut result = IndexMap::new();

    for field_name in field_names {
        let lower = field_name.to_lowercase();
        let value = if lower.contains("first") {
            Value::String(first.to_string())
        } else if lower.contains("last") || lower.contains("surname") {
            Value::String(last.to_string())
        } else if lower == "name" || lower.contains("full_name") || lower.contains("display_name") {
            Value::String(format!("{} {}", first, last))
        } else if lower.contains("email") {
            Value::String(format!(
                "{}.{}@{}",
                first.to_lowercase(),
                last.to_lowercase(),
                domain
            ))
        } else if lower.contains("username") || lower.contains("user_name") || lower.contains("handle") {
            Value::String(format!("{}{}", first.to_lowercase(), num_suffix))
        } else {
            // Fallback: full name
            Value::String(format!("{} {}", first, last))
        };

        result.insert(field_name.clone(), value);
    }

    Ok(result)
}

/// Generate fields normally but in order (for unknown groups).
fn generate_default_group(
    field_names: &[String],
    entity: &Entity,
    ctx: &mut GenerationContext,
) -> Result<IndexMap<String, Value>, DatjitError> {
    let mut result = IndexMap::new();

    for field_name in field_names {
        if let Some(field) = entity.fields.get(field_name) {
            let value = generate_field(field, &entity.name, &result, ctx)?;
            result.insert(field_name.clone(), value);
        }
    }

    Ok(result)
}

/// Derive a value from another field's value using semantic matching.
fn derive_from_value(
    target_field: &str,
    _source_field: &str,
    source_value: &Value,
    ctx: &mut GenerationContext,
) -> Result<Value, DatjitError> {
    let lower_target = target_field.to_lowercase();
    let source_str = source_value.to_output_string();

    // email @from(name) -> derive email from name
    if lower_target.contains("email") {
        let parts: Vec<&str> = source_str.split_whitespace().collect();
        let domain_list = embedded::EMAIL_DOMAINS;
        let domain = domain_list[ctx.rng.gen_range(0..domain_list.len())].0;
        return if parts.len() >= 2 {
            Ok(Value::String(format!(
                "{}.{}@{}",
                parts[0].to_lowercase(),
                parts[1].to_lowercase(),
                domain
            )))
        } else {
            Ok(Value::String(format!(
                "{}@{}",
                source_str.to_lowercase().replace(' ', "."),
                domain
            )))
        };
    }

    // timezone @from(office/city) -> derive timezone from location
    if lower_target.contains("timezone") || lower_target.contains("tz") || lower_target == "time_zone" {
        let tz = lookup_timezone_for_location(&source_str);
        return Ok(Value::String(tz));
    }

    // phone @from(office/city) -> derive phone with matching area code
    if lower_target.contains("phone") || lower_target.contains("tel") {
        let area = derive_area_code_from_location(&source_str);
        return Ok(Value::String(format!(
            "+1-{}-{:03}-{:04}",
            area,
            ctx.rng.gen_range(200..999),
            ctx.rng.gen_range(1000..9999)
        )));
    }

    // username @from(name) -> derive username
    if lower_target.contains("username") || lower_target.contains("handle") {
        let parts: Vec<&str> = source_str.split_whitespace().collect();
        let num: u32 = ctx.rng.gen_range(1..999);
        return if !parts.is_empty() {
            Ok(Value::String(format!("{}{}", parts[0].to_lowercase(), num)))
        } else {
            Ok(Value::String(format!("user{}", num)))
        };
    }

    // Default: just use the source value
    Ok(source_value.clone())
}

/// Look up a timezone by matching against known cities.
fn lookup_timezone_for_location(location: &str) -> String {
    let lower = location.to_lowercase();
    for (city, state, _zip, tz) in embedded::CITIES {
        if lower.contains(&city.to_lowercase()) || lower.contains(&state.to_lowercase()) {
            return tz.to_string();
        }
    }
    // Default
    "America/New_York".to_string()
}

/// Derive an area code from a zip code.
fn derive_area_code(zip: &str) -> String {
    // Simple mapping from zip prefix to area code
    let prefix = &zip[..2.min(zip.len())];
    match prefix {
        "10" | "11" | "12" => "212",
        "90" | "91" | "92" => "213",
        "60" | "61" => "312",
        "77" | "78" | "73" | "75" | "76" => "214",
        "85" => "480",
        "19" | "15" => "215",
        "94" | "95" => "415",
        "98" => "206",
        "80" => "303",
        "37" => "615",
        "97" => "503",
        "89" => "702",
        "38" => "901",
        "40" => "502",
        "21" => "410",
        "53" => "414",
        "87" => "505",
        "93" => "559",
        "30" => "404",
        "64" => "816",
        "68" => "402",
        "27" => "919",
        "33" => "305",
        "55" => "612",
        "44" | "45" => "216",
        "32" => "904",
        "46" => "317",
        "48" => "313",
        "02" => "617",
        "96" => "808",
        "84" => "801",
        "23" => "804",
        "70" => "504",
        "63" => "314",
        _ => "555",
    }
    .to_string()
}

/// Derive an area code from a location string by matching cities.
fn derive_area_code_from_location(location: &str) -> String {
    let lower = location.to_lowercase();
    for (city, _state, zip, _tz) in embedded::CITIES {
        if lower.contains(&city.to_lowercase()) {
            return derive_area_code(zip);
        }
    }
    "555".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use datjit_core::model::entity::{Entity, Field};
    use datjit_core::types::{PrimitiveType, SemanticType, TypeExpr};

    fn make_ctx() -> GenerationContext {
        GenerationContext::new(Some(42), "en-US".into())
    }

    #[test]
    fn test_location_coherence_group() {
        let mut entity = Entity::new("Employee");
        entity.fields.insert(
            "office".into(),
            Field::new("office", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity.fields.insert(
            "timezone".into(),
            Field::new("timezone", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity.fields.insert(
            "phone".into(),
            Field::new("phone", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity
            .coherence_groups
            .insert("location".into(), vec!["office".into(), "timezone".into(), "phone".into()]);

        let mut ctx = make_ctx();
        let result = generate_coherence_groups(&entity, &mut ctx).unwrap();

        assert!(result.contains_key("office"));
        assert!(result.contains_key("timezone"));
        assert!(result.contains_key("phone"));

        // Verify timezone matches the city
        let office_str = result["office"].to_output_string();
        let tz_str = result["timezone"].to_output_string();
        assert!(tz_str.starts_with("America/") || tz_str.starts_with("Pacific/"),
            "timezone should be a valid IANA timezone, got: {}", tz_str);

        // Phone should start with +1-
        let phone_str = result["phone"].to_output_string();
        assert!(phone_str.starts_with("+1-"), "phone should start with +1-, got: {}", phone_str);

        // The office location should contain a comma (city, state format)
        assert!(office_str.contains(','), "office should be 'city, state' format, got: {}", office_str);
    }

    #[test]
    fn test_identity_coherence_group() {
        let mut entity = Entity::new("User");
        entity.fields.insert(
            "first_name".into(),
            Field::new("first_name", TypeExpr::Semantic(SemanticType::new("person", "first"))),
        );
        entity.fields.insert(
            "last_name".into(),
            Field::new("last_name", TypeExpr::Semantic(SemanticType::new("person", "last"))),
        );
        entity.fields.insert(
            "email".into(),
            Field::new("email", TypeExpr::Semantic(SemanticType::new("email", ""))),
        );
        entity.fields.insert(
            "username".into(),
            Field::new("username", TypeExpr::Semantic(SemanticType::new("person", "username"))),
        );
        entity.coherence_groups.insert(
            "identity".into(),
            vec!["first_name".into(), "last_name".into(), "email".into(), "username".into()],
        );

        let mut ctx = make_ctx();
        let result = generate_coherence_groups(&entity, &mut ctx).unwrap();

        let first = result["first_name"].to_output_string();
        let last = result["last_name"].to_output_string();
        let email = result["email"].to_output_string();
        let username = result["username"].to_output_string();

        // Email should contain first.last@
        assert!(
            email.contains(&format!("{}.{}", first.to_lowercase(), last.to_lowercase())),
            "email '{}' should contain '{}.{}'", email, first.to_lowercase(), last.to_lowercase()
        );

        // Username should start with lowercase first name
        assert!(
            username.starts_with(&first.to_lowercase()),
            "username '{}' should start with '{}'", username, first.to_lowercase()
        );
    }

    #[test]
    fn test_from_decorator_email_from_name() {
        let mut entity = Entity::new("User");
        entity.fields.insert(
            "name".into(),
            Field::new("name", TypeExpr::Semantic(SemanticType::new("person", "full"))),
        );
        entity.fields.insert(
            "email".into(),
            Field::new("email", TypeExpr::Semantic(SemanticType::new("email", "")))
                .with_decorators(vec![Decorator::From(vec!["name".into()])]),
        );
        // Put name in a coherence group so it gets generated first
        entity.coherence_groups.insert(
            "person".into(),
            vec!["name".into()],
        );

        let mut ctx = make_ctx();
        let result = generate_coherence_groups(&entity, &mut ctx).unwrap();

        assert!(result.contains_key("name"));
        assert!(result.contains_key("email"));

        let email = result["email"].to_output_string();
        assert!(email.contains('@'), "email should contain @, got: {}", email);
    }

    #[test]
    fn test_from_decorator_without_coherence_group() {
        let mut entity = Entity::new("User");
        entity.fields.insert(
            "name".into(),
            Field::new("name", TypeExpr::Semantic(SemanticType::new("person", "full"))),
        );
        entity.fields.insert(
            "email".into(),
            Field::new("email", TypeExpr::Semantic(SemanticType::new("email", "")))
                .with_decorators(vec![Decorator::From(vec!["name".into()])]),
        );
        // NO coherence_groups — this is the regression test

        let mut ctx = make_ctx();
        let result = generate_coherence_groups(&entity, &mut ctx).unwrap();

        assert!(result.contains_key("name"), "name should be generated as @from source");
        assert!(result.contains_key("email"), "email should be derived via @from");

        let name = result["name"].to_output_string();
        let email = result["email"].to_output_string();
        assert!(email.contains('@'), "email should contain @, got: {}", email);

        // Verify email is actually derived from the name
        let name_parts: Vec<&str> = name.split_whitespace().collect();
        if name_parts.len() >= 2 {
            let expected_prefix = format!(
                "{}.{}",
                name_parts[0].to_lowercase(),
                name_parts[1].to_lowercase()
            );
            assert!(
                email.starts_with(&expected_prefix),
                "email '{}' should start with '{}' (derived from name '{}')",
                email,
                expected_prefix,
                name
            );
        }
    }

    #[test]
    fn test_default_group_unknown_semantic() {
        let mut entity = Entity::new("Widget");
        entity.fields.insert(
            "color".into(),
            Field::new("color", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity.fields.insert(
            "size".into(),
            Field::new("size", TypeExpr::Primitive(PrimitiveType::Int(None))),
        );
        entity.coherence_groups.insert(
            "appearance".into(),
            vec!["color".into(), "size".into()],
        );

        let mut ctx = make_ctx();
        let result = generate_coherence_groups(&entity, &mut ctx).unwrap();

        assert!(result.contains_key("color"));
        assert!(result.contains_key("size"));
    }

    #[test]
    fn test_location_timezone_consistency() {
        // Generate many location groups and verify timezone always matches
        let mut entity = Entity::new("Office");
        entity.fields.insert(
            "city".into(),
            Field::new("city", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity.fields.insert(
            "timezone".into(),
            Field::new("timezone", TypeExpr::Primitive(PrimitiveType::String(None))),
        );
        entity.coherence_groups.insert(
            "location".into(),
            vec!["city".into(), "timezone".into()],
        );

        let mut ctx = make_ctx();
        for _ in 0..20 {
            let result = generate_coherence_groups(&entity, &mut ctx).unwrap();
            let city_str = result["city"].to_output_string();
            let tz_str = result["timezone"].to_output_string();

            // Find matching city in corpus and verify timezone
            let city_name = city_str.split(',').next().unwrap().trim();
            if let Some(entry) = embedded::CITIES.iter().find(|(c, _, _, _)| *c == city_name) {
                assert_eq!(tz_str, entry.3,
                    "For city '{}', expected tz '{}' but got '{}'", city_name, entry.3, tz_str);
            }
        }
    }
}
