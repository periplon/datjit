use datjit_core::error::DatjitError;
use datjit_core::types::*;

/// Parse a type expression string into a TypeExpr.
///
/// Parse order:
/// 1. Union: `T1 | T2`
/// 2. Nullable: `T?`
/// 3. Compound: `[T]`, `{K: V}`, `(T1, T2)`
/// 4. Reference: `->Entity`, `->Entity?`, `->[Entity]`, `<->Entity`, `->self`
/// 5. Enum: `enum(a, b, c)`
/// 6. Parameterized primitive: `int(32)`, `decimal(10,2)`, `string(100)`
/// 7. Bare primitive: `string`, `int`, `float`, etc.
/// 8. Semantic: `person.full`, `email`, `address.city`
/// 9. Named type reference
pub fn parse_type(input: &str) -> Result<TypeExpr, DatjitError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(DatjitError::parse("type", "empty type expression"));
    }

    // 1. Check for union: T1 | T2 (split on | not inside brackets/parens)
    if let Some(types) = try_parse_union(input)? {
        return Ok(types);
    }

    // 2. Check for nullable: T?
    if input.ends_with('?') && !input.starts_with("->") {
        let inner = parse_type(&input[..input.len() - 1])?;
        return Ok(TypeExpr::Compound(CompoundType::Nullable(Box::new(inner))));
    }

    // 3. Compound types
    if input.starts_with('[') && input.ends_with(']') {
        let inner = &input[1..input.len() - 1].trim();
        let inner_type = parse_type(inner)?;
        return Ok(TypeExpr::Compound(CompoundType::List(Box::new(inner_type))));
    }

    if input.starts_with('{') && input.ends_with('}') {
        let inner = &input[1..input.len() - 1];
        if let Some((key, value)) = inner.split_once(':') {
            let key_type = parse_type(key.trim())?;
            let value_type = parse_type(value.trim())?;
            return Ok(TypeExpr::Compound(CompoundType::Map(
                Box::new(key_type),
                Box::new(value_type),
            )));
        }
    }

    if input.starts_with('(') && input.ends_with(')') {
        let inner = &input[1..input.len() - 1];
        let types = split_top_level(inner, ',')?
            .iter()
            .map(|s| parse_type(s.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(TypeExpr::Compound(CompoundType::Tuple(types)));
    }

    // 4. References
    if input.starts_with("<->") {
        let target = &input[3..].trim();
        return Ok(TypeExpr::Reference(ReferenceType::ManyToMany {
            target: target.to_string(),
        }));
    }

    if input.starts_with("->") {
        return parse_reference(&input[2..]);
    }

    // Has-many shorthand: [Entity] when not a generic list type
    // (This is already handled by the list case above, but Entity names are PascalCase)

    // 5. Enum: enum(a, b, c)
    if input.starts_with("enum(") && input.ends_with(')') {
        let inner = &input[5..input.len() - 1];
        let values: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
        return Ok(TypeExpr::Enum(EnumRef::Inline(values)));
    }

    // 6. Parameterized primitives
    if let Some(paren_start) = input.find('(') {
        if input.ends_with(')') {
            let name = &input[..paren_start];
            let params_str = &input[paren_start + 1..input.len() - 1];

            match name {
                "int" => {
                    let bits: u8 = params_str.trim().parse().map_err(|_| {
                        DatjitError::parse("type", format!("invalid bit width: {params_str}"))
                    })?;
                    return Ok(TypeExpr::Primitive(PrimitiveType::Int(Some(bits))));
                }
                "float" => {
                    let bits: u8 = params_str.trim().parse().map_err(|_| {
                        DatjitError::parse("type", format!("invalid bit width: {params_str}"))
                    })?;
                    return Ok(TypeExpr::Primitive(PrimitiveType::Float(Some(bits))));
                }
                "string" => {
                    let maxlen: usize = params_str.trim().parse().map_err(|_| {
                        DatjitError::parse("type", format!("invalid max length: {params_str}"))
                    })?;
                    return Ok(TypeExpr::Primitive(PrimitiveType::String(Some(maxlen))));
                }
                "bytes" => {
                    let maxlen: usize = params_str.trim().parse().map_err(|_| {
                        DatjitError::parse("type", format!("invalid max length: {params_str}"))
                    })?;
                    return Ok(TypeExpr::Primitive(PrimitiveType::Bytes(Some(maxlen))));
                }
                "decimal" => {
                    let parts: Vec<&str> = params_str.split(',').collect();
                    if parts.len() != 2 {
                        return Err(DatjitError::parse(
                            "type",
                            "decimal requires (precision, scale)",
                        ));
                    }
                    let precision: u8 = parts[0].trim().parse().map_err(|_| {
                        DatjitError::parse(
                            "type",
                            format!("invalid precision: {}", parts[0].trim()),
                        )
                    })?;
                    let scale: u8 = parts[1].trim().parse().map_err(|_| {
                        DatjitError::parse("type", format!("invalid scale: {}", parts[1].trim()))
                    })?;
                    return Ok(TypeExpr::Primitive(PrimitiveType::Decimal(
                        precision, scale,
                    )));
                }
                _ => {
                    // Could be a semantic type with params like currency(EUR) or text.paragraphs(3)
                    if let Some(mut st) = SemanticType::parse(name) {
                        st.params = params_str.split(',').map(|s| s.trim().to_string()).collect();
                        return Ok(TypeExpr::Semantic(st));
                    }
                }
            }
        }
    }

    // 7. Bare primitives
    if let Some(prim) = PrimitiveType::from_name(input) {
        return Ok(TypeExpr::Primitive(prim));
    }

    // 8. Semantic types
    if let Some(st) = SemanticType::parse(input) {
        return Ok(TypeExpr::Semantic(st));
    }

    // 9. Named type or enum reference (PascalCase = likely named type/enum)
    Ok(TypeExpr::Named(input.to_string()))
}

fn parse_reference(input: &str) -> Result<TypeExpr, DatjitError> {
    let input = input.trim();

    // ->self or ->self?
    if input == "self" {
        return Ok(TypeExpr::Reference(ReferenceType::SelfRef {
            optional: false,
        }));
    }
    if input == "self?" {
        return Ok(TypeExpr::Reference(ReferenceType::SelfRef {
            optional: true,
        }));
    }

    // ->[Entity]
    if input.starts_with('[') && input.ends_with(']') {
        let target = &input[1..input.len() - 1].trim();
        return Ok(TypeExpr::Reference(ReferenceType::HasMany {
            target: target.to_string(),
        }));
    }

    // ->Entity? (optional)
    if input.ends_with('?') {
        let target = &input[..input.len() - 1];
        return Ok(TypeExpr::Reference(ReferenceType::BelongsTo {
            target: target.to_string(),
            optional: true,
        }));
    }

    // ->Entity (required)
    Ok(TypeExpr::Reference(ReferenceType::BelongsTo {
        target: input.to_string(),
        optional: false,
    }))
}

fn try_parse_union(input: &str) -> Result<Option<TypeExpr>, DatjitError> {
    let parts = split_top_level(input, '|')?;
    if parts.len() <= 1 {
        return Ok(None);
    }

    // Check for polymorphic references: ->Post | ->Photo | ->Video
    let all_refs = parts.iter().all(|p| p.trim().starts_with("->"));
    if all_refs {
        let targets: Vec<String> = parts
            .iter()
            .map(|p| {
                let s = p.trim().trim_start_matches("->");
                s.to_string()
            })
            .collect();
        return Ok(Some(TypeExpr::Reference(ReferenceType::Polymorphic {
            targets,
        })));
    }

    let types = parts
        .iter()
        .map(|s| parse_type(s.trim()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(TypeExpr::Compound(CompoundType::Union(types))))
}

/// Split a string by a delimiter, but only at the top level
/// (not inside parentheses, brackets, or braces).
fn split_top_level(input: &str, delim: char) -> Result<Vec<String>, DatjitError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;

    for ch in input.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            c if c == delim && depth == 0 => {
                parts.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        assert_eq!(
            parse_type("string").unwrap(),
            TypeExpr::Primitive(PrimitiveType::String(None))
        );
        assert_eq!(
            parse_type("int").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Int(None))
        );
        assert_eq!(
            parse_type("float").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Float(None))
        );
        assert_eq!(
            parse_type("bool").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Bool)
        );
        assert_eq!(
            parse_type("datetime").unwrap(),
            TypeExpr::Primitive(PrimitiveType::DateTime)
        );
        assert_eq!(
            parse_type("uuid").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Uuid)
        );
    }

    #[test]
    fn test_parameterized_primitives() {
        assert_eq!(
            parse_type("int(32)").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Int(Some(32)))
        );
        assert_eq!(
            parse_type("string(100)").unwrap(),
            TypeExpr::Primitive(PrimitiveType::String(Some(100)))
        );
        assert_eq!(
            parse_type("decimal(10, 2)").unwrap(),
            TypeExpr::Primitive(PrimitiveType::Decimal(10, 2))
        );
    }

    #[test]
    fn test_semantic_types() {
        let result = parse_type("person.full").unwrap();
        match result {
            TypeExpr::Semantic(st) => {
                assert_eq!(st.namespace, "person");
                assert_eq!(st.tag, "full");
            }
            _ => panic!("expected Semantic"),
        }

        let result = parse_type("email").unwrap();
        match result {
            TypeExpr::Semantic(st) => {
                assert_eq!(st.namespace, "email");
                assert_eq!(st.tag, "");
            }
            _ => panic!("expected Semantic"),
        }
    }

    #[test]
    fn test_enum_inline() {
        let result = parse_type("enum(active, inactive, suspended)").unwrap();
        match result {
            TypeExpr::Enum(EnumRef::Inline(values)) => {
                assert_eq!(values, vec!["active", "inactive", "suspended"]);
            }
            _ => panic!("expected Enum::Inline"),
        }
    }

    #[test]
    fn test_references() {
        // Required
        let result = parse_type("->User").unwrap();
        assert_eq!(
            result,
            TypeExpr::Reference(ReferenceType::BelongsTo {
                target: "User".into(),
                optional: false
            })
        );

        // Optional
        let result = parse_type("->User?").unwrap();
        assert_eq!(
            result,
            TypeExpr::Reference(ReferenceType::BelongsTo {
                target: "User".into(),
                optional: true
            })
        );

        // Self-ref
        assert_eq!(
            parse_type("->self").unwrap(),
            TypeExpr::Reference(ReferenceType::SelfRef { optional: false })
        );
        assert_eq!(
            parse_type("->self?").unwrap(),
            TypeExpr::Reference(ReferenceType::SelfRef { optional: true })
        );

        // Has-many
        assert_eq!(
            parse_type("->[Tag]").unwrap(),
            TypeExpr::Reference(ReferenceType::HasMany {
                target: "Tag".into()
            })
        );

        // Many-to-many
        assert_eq!(
            parse_type("<->Tag").unwrap(),
            TypeExpr::Reference(ReferenceType::ManyToMany {
                target: "Tag".into()
            })
        );
    }

    #[test]
    fn test_compound_list() {
        let result = parse_type("[int]").unwrap();
        assert_eq!(
            result,
            TypeExpr::Compound(CompoundType::List(Box::new(TypeExpr::Primitive(
                PrimitiveType::Int(None)
            ))))
        );
    }

    #[test]
    fn test_compound_map() {
        let result = parse_type("{string: int}").unwrap();
        match result {
            TypeExpr::Compound(CompoundType::Map(k, v)) => {
                assert_eq!(*k, TypeExpr::Primitive(PrimitiveType::String(None)));
                assert_eq!(*v, TypeExpr::Primitive(PrimitiveType::Int(None)));
            }
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn test_nullable() {
        let result = parse_type("string?").unwrap();
        assert_eq!(
            result,
            TypeExpr::Compound(CompoundType::Nullable(Box::new(TypeExpr::Primitive(
                PrimitiveType::String(None)
            ))))
        );
    }

    #[test]
    fn test_union() {
        let result = parse_type("string | int").unwrap();
        match result {
            TypeExpr::Compound(CompoundType::Union(types)) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("expected Union"),
        }
    }

    #[test]
    fn test_polymorphic_reference() {
        let result = parse_type("->Post | ->Photo | ->Video").unwrap();
        match result {
            TypeExpr::Reference(ReferenceType::Polymorphic { targets }) => {
                assert_eq!(targets, vec!["Post", "Photo", "Video"]);
            }
            _ => panic!("expected Polymorphic"),
        }
    }

    #[test]
    fn test_named_type() {
        let result = parse_type("Address").unwrap();
        assert_eq!(result, TypeExpr::Named("Address".into()));
    }

    #[test]
    fn test_semantic_with_params() {
        let result = parse_type("currency(EUR)").unwrap();
        match result {
            TypeExpr::Semantic(st) => {
                assert_eq!(st.namespace, "currency");
                assert_eq!(st.params, vec!["EUR"]);
            }
            _ => panic!("expected Semantic with params"),
        }
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_type("").is_err());
        assert!(parse_type("   ").is_err());
    }
}
