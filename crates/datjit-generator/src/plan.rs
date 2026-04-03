use std::collections::{HashMap, HashSet, VecDeque};

use datjit_core::error::DatjitError;
use datjit_core::model::{DdlDocument, VolumeSpec};
use datjit_core::types::{ReferenceType, TypeExpr};

/// A plan for generating data: entity order and volumes.
pub struct GenerationPlan {
    /// Entities in dependency order (dependencies come first).
    pub entity_order: Vec<String>,
    /// Volume per entity.
    pub volumes: HashMap<String, usize>,
}

impl GenerationPlan {
    pub fn from_document(doc: &DdlDocument) -> Result<Self, DatjitError> {
        let dependencies = build_dependency_graph(doc);
        let entity_order = topological_sort(&dependencies, doc)?;
        let volumes = resolve_volumes(doc);

        Ok(Self {
            entity_order,
            volumes,
        })
    }
}

/// Build a dependency graph: entity -> set of entities it depends on.
fn build_dependency_graph(doc: &DdlDocument) -> HashMap<String, HashSet<String>> {
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();

    for (name, entity) in &doc.entities {
        let entry = deps.entry(name.clone()).or_default();
        for field in entity.fields.values() {
            match &field.type_expr {
                TypeExpr::Reference(ReferenceType::BelongsTo { target, .. }) => {
                    if doc.entities.contains_key(target) {
                        entry.insert(target.clone());
                    }
                }
                TypeExpr::Reference(ReferenceType::ManyToMany { target }) => {
                    if doc.entities.contains_key(target) {
                        entry.insert(target.clone());
                    }
                }
                _ => {}
            }
        }
        // Self-references don't count as external dependencies
        entry.remove(name);
    }

    deps
}

/// Topological sort using Kahn's algorithm.
fn topological_sort(
    deps: &HashMap<String, HashSet<String>>,
    doc: &DdlDocument,
) -> Result<Vec<String>, DatjitError> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize
    for name in doc.entities.keys() {
        in_degree.entry(name.clone()).or_insert(0);
    }

    for (name, dep_set) in deps {
        *in_degree.entry(name.clone()).or_insert(0) += dep_set.len();
        for dep in dep_set {
            reverse_deps
                .entry(dep.clone())
                .or_default()
                .push(name.clone());
        }
    }

    // Collect zero-degree nodes and sort them to ensure deterministic ordering.
    // Use the document's entity definition order as the tie-breaker so that
    // the generation order is reproducible across runs.
    let doc_order: Vec<String> = doc.entities.keys().cloned().collect();
    let mut zero_degree: Vec<String> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(n, _)| n.clone())
        .collect();
    zero_degree.sort_by_key(|n| doc_order.iter().position(|o| o == n).unwrap_or(usize::MAX));
    let mut queue: VecDeque<String> = zero_degree.into_iter().collect();

    let mut order = Vec::new();
    while let Some(name) = queue.pop_front() {
        order.push(name.clone());
        if let Some(dependents) = reverse_deps.get(&name) {
            // Collect newly-ready dependents and sort by document order for determinism
            let mut newly_ready = Vec::new();
            for dep in dependents {
                if let Some(degree) = in_degree.get_mut(dep) {
                    *degree -= 1;
                    if *degree == 0 {
                        newly_ready.push(dep.clone());
                    }
                }
            }
            newly_ready
                .sort_by_key(|n| doc_order.iter().position(|o| o == n).unwrap_or(usize::MAX));
            for dep in newly_ready {
                queue.push_back(dep);
            }
        }
    }

    if order.len() != doc.entities.len() {
        let missing: Vec<String> = doc
            .entities
            .keys()
            .filter(|k| !order.contains(k))
            .cloned()
            .collect();
        return Err(DatjitError::CircularDependency(format!(
            "circular dependency involving: {}",
            missing.join(", ")
        )));
    }

    Ok(order)
}

/// Resolve volumes: explicit > default (100).
fn resolve_volumes(doc: &DdlDocument) -> HashMap<String, usize> {
    let mut volumes = HashMap::new();
    for name in doc.entities.keys() {
        let vol = doc
            .volume
            .get(name)
            .map(|v| match v {
                VolumeSpec::Exact(n) => *n,
                VolumeSpec::Range(lo, hi) => (*lo + *hi) / 2,
                VolumeSpec::Inferred => 100,
            })
            .unwrap_or(100);
        volumes.insert(name.clone(), vol);
    }
    volumes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc_with_ref() -> DdlDocument {
        use datjit_core::model::{Entity, Field};
        use datjit_core::types::PrimitiveType;

        let mut doc = DdlDocument::new("test");

        let mut user = Entity::new("User");
        user.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid)),
        );

        let mut order = Entity::new("Order");
        order.fields.insert(
            "id".into(),
            Field::new("id", TypeExpr::Primitive(PrimitiveType::Uuid)),
        );
        order.fields.insert(
            "user".into(),
            Field::new(
                "user",
                TypeExpr::Reference(ReferenceType::BelongsTo {
                    target: "User".into(),
                    optional: false,
                }),
            ),
        );

        doc.entities.insert("Order".into(), order);
        doc.entities.insert("User".into(), user);
        doc
    }

    #[test]
    fn test_topological_sort() {
        let doc = make_doc_with_ref();
        let plan = GenerationPlan::from_document(&doc).unwrap();
        // User must come before Order
        let user_idx = plan.entity_order.iter().position(|n| n == "User").unwrap();
        let order_idx = plan.entity_order.iter().position(|n| n == "Order").unwrap();
        assert!(user_idx < order_idx);
    }

    #[test]
    fn test_default_volumes() {
        let doc = make_doc_with_ref();
        let plan = GenerationPlan::from_document(&doc).unwrap();
        assert_eq!(plan.volumes["User"], 100);
        assert_eq!(plan.volumes["Order"], 100);
    }
}
