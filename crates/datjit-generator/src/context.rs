use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use datjit_core::value::Value;
use datjit_corpus::CorpusRegistry;

/// Mutable state during data generation.
pub struct GenerationContext {
    pub rng: ChaCha8Rng,
    pub locale: String,
    /// Already-generated rows per entity.
    pub generated: IndexMap<String, Vec<IndexMap<String, Value>>>,
    /// Unique value sets: (entity_name, field_name) -> set of used values.
    pub unique_sets: HashMap<(String, String), HashSet<Value>>,
    /// Auto-increment counters per entity.
    pub counters: HashMap<String, u64>,
    /// Corpus registry for semantic type generation.
    pub corpus: CorpusRegistry,
}

impl GenerationContext {
    pub fn new(seed: Option<u64>, locale: String) -> Self {
        let rng = match seed {
            Some(s) => ChaCha8Rng::seed_from_u64(s),
            None => ChaCha8Rng::from_entropy(),
        };
        let corpus = CorpusRegistry::new(&locale);
        Self {
            rng,
            locale,
            generated: IndexMap::new(),
            unique_sets: HashMap::new(),
            counters: HashMap::new(),
            corpus,
        }
    }

    /// Get the next auto-increment value for an entity.
    pub fn next_counter(&mut self, entity: &str) -> u64 {
        let counter = self.counters.entry(entity.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Check if a value is unique for the given entity/field, and insert it.
    pub fn check_and_insert_unique(
        &mut self,
        entity: &str,
        field: &str,
        value: &Value,
    ) -> bool {
        let key = (entity.to_string(), field.to_string());
        let set = self.unique_sets.entry(key).or_default();
        set.insert(value.clone())
    }

    /// Get all generated rows for an entity.
    pub fn entity_rows(&self, entity: &str) -> &[IndexMap<String, Value>] {
        self.generated
            .get(entity)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
