use super::decorator::Decorator;
use super::entity::{Entity, Field};
use crate::types::{PrimitiveType, TypeExpr};

/// The full set of inferred CRUD tools for an entity.
#[derive(Debug, Clone)]
pub struct InferredTools {
    pub entity_name: String,
    pub list: Option<ListTool>,
    pub get: Option<GetTool>,
    pub create: Option<CreateTool>,
    pub update: Option<UpdateTool>,
    pub delete: Option<DeleteTool>,
    /// Custom MCP tools declared in the DDL schema.
    pub custom_tools: Vec<super::mcp_tool::McpToolDef>,
    /// Trigger metadata for field change propagation.
    pub triggers: Vec<TriggerInfo>,
}

/// Trigger metadata exported for MCP clients.
#[derive(Debug, Clone)]
pub struct TriggerInfo {
    /// Fields that fire this trigger.
    pub on_fields: Vec<String>,
    /// Fields that get recomputed.
    pub recomputed_fields: Vec<String>,
    /// Rules that get validated.
    pub validated_rules: Vec<String>,
}

/// List tool with filtering, sorting, searching, and pagination.
#[derive(Debug, Clone)]
pub struct ListTool {
    pub filters: Vec<String>,
    pub sorts: Vec<String>,
    pub search_fields: Vec<String>,
    pub page_size: usize,
}

/// Get-by-id tool.
#[derive(Debug, Clone)]
pub struct GetTool {
    pub primary_key: String,
}

/// Create tool with required and optional fields.
#[derive(Debug, Clone)]
pub struct CreateTool {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

/// Update tool with mutable fields.
#[derive(Debug, Clone)]
pub struct UpdateTool {
    pub mutable_fields: Vec<String>,
}

/// Delete tool with strategy.
#[derive(Debug, Clone)]
pub struct DeleteTool {
    pub strategy: String,
}

/// Infer the CRUD tool surface for an entity based on its definition and decorators.
pub fn infer_tools(entity: &Entity) -> InferredTools {
    let is_readonly = entity.is_readonly();
    let is_immutable = entity.is_immutable();
    let has_no_delete = entity.meta.iter().any(|d| matches!(d, Decorator::NoDelete));
    let has_soft_delete = entity
        .meta
        .iter()
        .any(|d| matches!(d, Decorator::SoftDelete));

    let pk_name = entity
        .primary_key()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "id".into());

    // Page size from @paginated on the entity meta, default 25
    let page_size = entity
        .meta
        .iter()
        .find_map(|d| match d {
            Decorator::Paginated(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(25);

    // List tool: always present
    let list = Some(infer_list_tool(entity, page_size));

    // Get tool: always present
    let get = Some(GetTool {
        primary_key: pk_name,
    });

    // Create tool: not if readonly
    let create = if is_readonly {
        None
    } else {
        Some(infer_create_tool(entity))
    };

    // Update tool: not if readonly or immutable
    let update = if is_readonly || is_immutable {
        None
    } else {
        Some(infer_update_tool(entity))
    };

    // Delete tool: not if readonly, immutable, or @no_delete
    let delete = if is_readonly || is_immutable || has_no_delete {
        None
    } else {
        let strategy = if has_soft_delete {
            "soft".into()
        } else {
            "hard".into()
        };
        Some(DeleteTool { strategy })
    };

    // Build trigger info from entity triggers
    let triggers: Vec<TriggerInfo> = entity
        .triggers
        .iter()
        .map(|t| TriggerInfo {
            on_fields: t.on.clone(),
            recomputed_fields: t.recompute.clone(),
            validated_rules: t.validate.clone(),
        })
        .collect();

    InferredTools {
        entity_name: entity.name.clone(),
        list,
        get,
        create,
        update,
        delete,
        custom_tools: Vec::new(),
        triggers,
    }
}

fn infer_list_tool(entity: &Entity, page_size: usize) -> ListTool {
    let mut filters = Vec::new();
    let mut sorts = Vec::new();
    let mut search_fields = Vec::new();

    for field in entity.fields.values() {
        // Skip @write_only fields from list output
        if field
            .decorators
            .iter()
            .any(|d| matches!(d, Decorator::WriteOnly))
        {
            continue;
        }

        // Filters
        if is_filterable(field) {
            filters.push(field.name.clone());

            // Date/datetime fields get _after/_before virtual filters
            if is_date_type(&field.type_expr) {
                filters.push(format!("{}_after", field.name));
                filters.push(format!("{}_before", field.name));
            }
        }

        // Sorts
        if is_sortable(field) {
            sorts.push(field.name.clone());
        }

        // Search
        if field
            .decorators
            .iter()
            .any(|d| matches!(d, Decorator::Searchable))
        {
            search_fields.push(field.name.clone());
        }
    }

    ListTool {
        filters,
        sorts,
        search_fields,
        page_size,
    }
}

fn is_filterable(field: &Field) -> bool {
    // Explicitly @filterable
    if field
        .decorators
        .iter()
        .any(|d| matches!(d, Decorator::Filterable))
    {
        return true;
    }
    // @index fields
    if field
        .decorators
        .iter()
        .any(|d| matches!(d, Decorator::Index))
    {
        return true;
    }
    // Enum fields
    if matches!(field.type_expr, TypeExpr::Enum(_)) {
        return true;
    }
    // Reference fields
    if matches!(field.type_expr, TypeExpr::Reference(_)) {
        return true;
    }
    // Bool fields
    if matches!(field.type_expr, TypeExpr::Primitive(PrimitiveType::Bool)) {
        return true;
    }
    // Date/datetime fields
    if is_date_type(&field.type_expr) {
        return true;
    }
    false
}

fn is_sortable(field: &Field) -> bool {
    // Explicitly @sortable
    if field
        .decorators
        .iter()
        .any(|d| matches!(d, Decorator::Sortable))
    {
        return true;
    }
    // @primary fields
    if field.is_primary() {
        return true;
    }
    // Date/datetime fields
    if is_date_type(&field.type_expr) {
        return true;
    }
    // Numeric @index fields
    if field
        .decorators
        .iter()
        .any(|d| matches!(d, Decorator::Index))
        && is_numeric_type(&field.type_expr)
    {
        return true;
    }
    false
}

fn is_date_type(t: &TypeExpr) -> bool {
    matches!(
        t,
        TypeExpr::Primitive(PrimitiveType::Date | PrimitiveType::DateTime)
    )
}

fn is_numeric_type(t: &TypeExpr) -> bool {
    matches!(
        t,
        TypeExpr::Primitive(
            PrimitiveType::Int(_) | PrimitiveType::Float(_) | PrimitiveType::Decimal(_, _)
        )
    )
}

fn infer_create_tool(entity: &Entity) -> CreateTool {
    let mut required_fields = Vec::new();
    let mut optional_fields = Vec::new();

    for field in entity.fields.values() {
        // Skip @auto, @primary, @derived
        if field.is_auto() || field.is_primary() || field.is_derived() {
            continue;
        }

        // @optional or @default fields are optional, rest required
        if field.is_optional()
            || field
                .decorators
                .iter()
                .any(|d| matches!(d, Decorator::Default(_)))
        {
            optional_fields.push(field.name.clone());
        } else {
            required_fields.push(field.name.clone());
        }
    }

    CreateTool {
        required_fields,
        optional_fields,
    }
}

fn infer_update_tool(entity: &Entity) -> UpdateTool {
    let mut mutable_fields = Vec::new();

    for field in entity.fields.values() {
        // Skip @auto, @primary, @immutable, @derived
        if field.is_auto() || field.is_primary() || field.is_derived() {
            continue;
        }
        if field
            .decorators
            .iter()
            .any(|d| matches!(d, Decorator::Immutable))
        {
            continue;
        }
        mutable_fields.push(field.name.clone());
    }

    UpdateTool { mutable_fields }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::decorator::{Expression, FieldPath, LiteralValue};
    use crate::types::type_expr::EnumRef;
    use indexmap::IndexMap;

    fn make_entity(name: &str, meta: Vec<Decorator>, fields: Vec<Field>) -> Entity {
        let mut field_map = IndexMap::new();
        for f in fields {
            field_map.insert(f.name.clone(), f);
        }
        Entity {
            name: name.into(),
            meta,
            fields: field_map,
            coherence_groups: IndexMap::new(),
            triggers: Vec::new(),
        }
    }

    #[test]
    fn test_readonly_entity() {
        let entity = make_entity(
            "AuditLog",
            vec![Decorator::Readonly],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("message", TypeExpr::Primitive(PrimitiveType::String(None))),
            ],
        );

        let tools = infer_tools(&entity);
        assert_eq!(tools.entity_name, "AuditLog");
        assert!(tools.list.is_some());
        assert!(tools.get.is_some());
        assert!(tools.create.is_none());
        assert!(tools.update.is_none());
        assert!(tools.delete.is_none());
    }

    #[test]
    fn test_immutable_entity() {
        let entity = make_entity(
            "Event",
            vec![Decorator::Immutable],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("payload", TypeExpr::Primitive(PrimitiveType::String(None))),
            ],
        );

        let tools = infer_tools(&entity);
        assert!(tools.list.is_some());
        assert!(tools.get.is_some());
        assert!(tools.create.is_some());
        assert!(tools.update.is_none());
        assert!(tools.delete.is_none());
    }

    #[test]
    fn test_normal_entity_full_crud() {
        let entity = make_entity(
            "User",
            vec![],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("email", TypeExpr::Primitive(PrimitiveType::String(None)))
                    .with_decorators(vec![Decorator::Unique, Decorator::Searchable]),
                Field::new("name", TypeExpr::Primitive(PrimitiveType::String(None)))
                    .with_decorators(vec![Decorator::Searchable]),
                Field::new("is_active", TypeExpr::Primitive(PrimitiveType::Bool)),
                Field::new("created_at", TypeExpr::Primitive(PrimitiveType::DateTime))
                    .with_decorators(vec![Decorator::Auto]),
                Field::new(
                    "status",
                    TypeExpr::Enum(EnumRef::Inline(vec!["active".into(), "inactive".into()])),
                ),
                Field::new("bio", TypeExpr::Primitive(PrimitiveType::String(None)))
                    .with_decorators(vec![Decorator::Optional]),
                Field::new("score", TypeExpr::Primitive(PrimitiveType::Int(None))).with_decorators(
                    vec![Decorator::Index, Decorator::Default(LiteralValue::Int(0))],
                ),
            ],
        );

        let tools = infer_tools(&entity);

        // All CRUD tools should be present
        assert!(tools.list.is_some());
        assert!(tools.get.is_some());
        assert!(tools.create.is_some());
        assert!(tools.update.is_some());
        assert!(tools.delete.is_some());

        let list = tools.list.unwrap();
        // is_active (bool), created_at (datetime), status (enum), score (@index) should be filterable
        assert!(list.filters.contains(&"is_active".into()));
        assert!(list.filters.contains(&"created_at".into()));
        assert!(list.filters.contains(&"created_at_after".into()));
        assert!(list.filters.contains(&"created_at_before".into()));
        assert!(list.filters.contains(&"status".into()));
        assert!(list.filters.contains(&"score".into()));

        // Search fields
        assert!(list.search_fields.contains(&"email".into()));
        assert!(list.search_fields.contains(&"name".into()));

        // Sorts: id (@primary), created_at (datetime), score (numeric @index)
        assert!(list.sorts.contains(&"id".into()));
        assert!(list.sorts.contains(&"created_at".into()));
        assert!(list.sorts.contains(&"score".into()));

        // Create: @auto and @primary excluded
        let create = tools.create.unwrap();
        assert!(create.required_fields.contains(&"email".into()));
        assert!(create.required_fields.contains(&"name".into()));
        assert!(create.required_fields.contains(&"status".into()));
        assert!(create.required_fields.contains(&"is_active".into()));
        // @optional and @default fields are optional
        assert!(create.optional_fields.contains(&"bio".into()));
        assert!(create.optional_fields.contains(&"score".into()));
        // @auto field excluded entirely
        assert!(!create.required_fields.contains(&"created_at".into()));
        assert!(!create.optional_fields.contains(&"created_at".into()));
        // @primary field excluded entirely
        assert!(!create.required_fields.contains(&"id".into()));

        // Update: exclude @auto, @primary, @immutable, @derived
        let update = tools.update.unwrap();
        assert!(update.mutable_fields.contains(&"email".into()));
        assert!(update.mutable_fields.contains(&"name".into()));
        assert!(!update.mutable_fields.contains(&"id".into()));
        assert!(!update.mutable_fields.contains(&"created_at".into()));

        // Delete: hard (no @soft_delete)
        let delete = tools.delete.unwrap();
        assert_eq!(delete.strategy, "hard");
    }

    #[test]
    fn test_soft_delete_entity() {
        let entity = make_entity(
            "Post",
            vec![Decorator::SoftDelete],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("title", TypeExpr::Primitive(PrimitiveType::String(None))),
            ],
        );

        let tools = infer_tools(&entity);
        let delete = tools.delete.unwrap();
        assert_eq!(delete.strategy, "soft");
    }

    #[test]
    fn test_no_delete_entity() {
        let entity = make_entity(
            "LegalRecord",
            vec![Decorator::NoDelete],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("content", TypeExpr::Primitive(PrimitiveType::String(None))),
            ],
        );

        let tools = infer_tools(&entity);
        assert!(tools.list.is_some());
        assert!(tools.get.is_some());
        assert!(tools.create.is_some());
        assert!(tools.update.is_some());
        assert!(tools.delete.is_none());
    }

    #[test]
    fn test_derived_and_immutable_fields() {
        let entity = make_entity(
            "OrderLine",
            vec![],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("qty", TypeExpr::Primitive(PrimitiveType::Int(None))),
                Field::new("price", TypeExpr::Primitive(PrimitiveType::Decimal(10, 2)))
                    .with_decorators(vec![Decorator::Immutable]),
                Field::new("total", TypeExpr::Primitive(PrimitiveType::Decimal(10, 2)))
                    .with_decorators(vec![Decorator::Derived(Expression::FieldRef(
                        FieldPath::parse("qty"),
                    ))]),
            ],
        );

        let tools = infer_tools(&entity);

        // Create: derived excluded, immutable field is still included
        let create = tools.create.unwrap();
        assert!(create.required_fields.contains(&"qty".into()));
        assert!(create.required_fields.contains(&"price".into()));
        assert!(!create.required_fields.contains(&"total".into()));
        assert!(!create.optional_fields.contains(&"total".into()));

        // Update: @immutable and @derived fields excluded
        let update = tools.update.unwrap();
        assert!(update.mutable_fields.contains(&"qty".into()));
        assert!(!update.mutable_fields.contains(&"price".into()));
        assert!(!update.mutable_fields.contains(&"total".into()));
    }

    #[test]
    fn test_paginated_entity() {
        let entity = make_entity(
            "Product",
            vec![Decorator::Paginated(50)],
            vec![
                Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid))
                    .with_decorators(vec![Decorator::Primary]),
                Field::new("name", TypeExpr::Primitive(PrimitiveType::String(None)))
                    .with_decorators(vec![Decorator::Sortable]),
            ],
        );

        let tools = infer_tools(&entity);
        let list = tools.list.unwrap();
        assert_eq!(list.page_size, 50);
        assert!(list.sorts.contains(&"name".into()));
        assert!(list.sorts.contains(&"id".into()));
    }
}
