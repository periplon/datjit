use datjit_core::error::DatjitError;
use datjit_core::ports::corpus::CorpusProvider;
use datjit_core::types::SemanticType;
use datjit_core::value::Value;
use rand::Rng;

use crate::embedded;

/// Corpus registry that provides data for semantic type generation.
/// Currently uses the embedded minimal corpus; will be extended to load
/// external JSON corpus files.
pub struct CorpusRegistry {
    locale: String,
}

impl CorpusRegistry {
    pub fn new(locale: &str) -> Self {
        Self {
            locale: locale.to_string(),
        }
    }
}

impl CorpusProvider for CorpusRegistry {
    fn sample(
        &self,
        semantic: &SemanticType,
        rng: &mut dyn rand::RngCore,
    ) -> Result<Value, DatjitError> {
        let full = semantic.full_name();
        let val = match full.as_str() {
            "person.full" => {
                let first = sample_first_name(rng);
                let last = pick(embedded::LAST_NAMES, rng);
                Value::String(format!("{first} {last}"))
            }
            "person.first" => Value::String(sample_first_name(rng).to_string()),
            "person.last" => Value::String(pick(embedded::LAST_NAMES, rng).to_string()),
            "person.username" => {
                let first = sample_first_name(rng).to_lowercase();
                let num = rng.gen_range(1..999);
                Value::String(format!("{first}{num}"))
            }
            "email" => {
                let first = sample_first_name(rng).to_lowercase();
                let last = pick(embedded::LAST_NAMES, rng).to_lowercase();
                let domain = pick_email_domain(rng);
                Value::String(format!("{first}.{last}@{domain}"))
            }
            "company.name" => {
                let prefix = pick(embedded::COMPANY_PREFIXES, rng);
                let core = pick(embedded::COMPANY_CORES, rng);
                let suffix = pick(embedded::COMPANY_SUFFIXES, rng);
                Value::String(format!("{prefix} {core} {suffix}"))
            }
            "job.title" => {
                let (title, _) = embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                Value::String(title.to_string())
            }
            "job.department" => {
                let (_, dept) = embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                Value::String(dept.to_string())
            }
            "address.full" => {
                let (city, state, zip, _) = pick_city(rng);
                let num = rng.gen_range(100..9999);
                let street = pick(embedded::STREET_NAMES, rng);
                let suffix = pick(embedded::STREET_SUFFIXES, rng);
                Value::String(format!("{num} {street} {suffix}, {city}, {state} {zip}"))
            }
            "address.street" => {
                let num = rng.gen_range(100..9999);
                let street = pick(embedded::STREET_NAMES, rng);
                let suffix = pick(embedded::STREET_SUFFIXES, rng);
                Value::String(format!("{num} {street} {suffix}"))
            }
            "address.city" => {
                let (city, _, _, _) = pick_city(rng);
                Value::String(city.to_string())
            }
            "address.state" => {
                let (_, state, _, _) = pick_city(rng);
                Value::String(state.to_string())
            }
            "address.zip" => {
                let (_, _, zip, _) = pick_city(rng);
                Value::String(zip.to_string())
            }
            "address.country" => Value::String("US".into()),
            "timezone" => {
                let (_, _, _, tz) = pick_city(rng);
                Value::String(tz.to_string())
            }
            _ => {
                return Err(DatjitError::Corpus(format!(
                    "no corpus data for semantic type: {full}"
                )));
            }
        };
        Ok(val)
    }

    fn available_locales(&self) -> Vec<String> {
        vec!["en-US".into()]
    }

    fn set_locale(&mut self, locale: &str) -> Result<(), DatjitError> {
        self.locale = locale.to_string();
        Ok(())
    }
}

fn sample_first_name(rng: &mut dyn rand::RngCore) -> &'static str {
    if rng.gen_bool(0.5) {
        pick(embedded::FIRST_NAMES_MALE, rng)
    } else {
        pick(embedded::FIRST_NAMES_FEMALE, rng)
    }
}

fn pick<'a>(items: &'a [&str], rng: &mut dyn rand::RngCore) -> &'a str {
    items[rng.gen_range(0..items.len())]
}

fn pick_city(rng: &mut dyn rand::RngCore) -> &'static (&'static str, &'static str, &'static str, &'static str) {
    &embedded::CITIES[rng.gen_range(0..embedded::CITIES.len())]
}

fn pick_email_domain(rng: &mut dyn rand::RngCore) -> &'static str {
    let total: f64 = embedded::EMAIL_DOMAINS.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0.0..total);
    for (domain, weight) in embedded::EMAIL_DOMAINS {
        roll -= weight;
        if roll <= 0.0 {
            return domain;
        }
    }
    embedded::EMAIL_DOMAINS.last().unwrap().0
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
    fn test_person_full() {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new("person", "full");
        let val = registry.sample(&st, &mut rng()).unwrap();
        match val {
            Value::String(s) => assert!(s.contains(' '), "should have first and last name"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_email() {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new("email", "");
        let val = registry.sample(&st, &mut rng()).unwrap();
        match val {
            Value::String(s) => assert!(s.contains('@'), "should contain @"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_company_name() {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new("company", "name");
        let val = registry.sample(&st, &mut rng()).unwrap();
        assert!(matches!(val, Value::String(_)));
    }

    #[test]
    fn test_unknown_semantic() {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new("unknown", "type");
        assert!(registry.sample(&st, &mut rng()).is_err());
    }
}
