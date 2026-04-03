use std::collections::HashMap;
use std::path::PathBuf;

use datjit_core::error::DatjitError;
use datjit_core::ports::corpus::CorpusProvider;
use datjit_core::types::SemanticType;
use datjit_core::value::Value;
use rand::Rng;

use crate::embedded;

/// Corpus registry that provides data for semantic type generation.
/// Loads external JSON corpus files from `~/.datjit/corpus/` when available,
/// falling back to the embedded minimal corpus.
pub struct CorpusRegistry {
    locale: String,
    #[allow(dead_code)]
    corpus_dir: Option<PathBuf>,
    cache: HashMap<String, serde_json::Value>,
}

/// List of corpus files to preload when the corpus directory exists.
const CORPUS_FILES: &[&str] = &[
    "en-US/person_first.json",
    "en-US/person_last.json",
    "en-US/cities.json",
    "en-US/postal_codes.json",
    "shared/countries.json",
    "shared/admin1.json",
    "shared/timezones.json",
    "shared/job_titles.json",
    "shared/companies.json",
    "shared/products.json",
    "shared/currencies.json",
    "shared/credit_card_bins.json",
    "shared/phone_formats.json",
    "shared/mime_types.json",
    "shared/mac_vendors.json",
    "shared/color_names.json",
    "shared/airports.json",
    "shared/airlines.json",
    "shared/tlds.json",
    "shared/accounting_plans.json",
    // Odoo ERP reference data
    "shared/erp_countries.json",
    "shared/erp_states.json",
    "shared/erp_currencies.json",
    "shared/erp_uom.json",
    "shared/erp_payment_terms.json",
    "shared/erp_incoterms.json",
    "shared/erp_tax_rates.json",
    "shared/erp_account_types.json",
    // Ecommerce datasets
    "shared/instacart_aisles.json",
    "shared/instacart_departments.json",
    "shared/instacart_products.json",
    "shared/uk_retail_products.json",
];

impl CorpusRegistry {
    pub fn new(locale: &str) -> Self {
        let corpus_dir = crate::updater::default_corpus_dir();
        let mut cache = HashMap::new();
        let dir_exists = corpus_dir.exists();
        if dir_exists {
            for file in CORPUS_FILES {
                let path = corpus_dir.join(file);
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(value) = serde_json::from_str(&data) {
                        cache.insert(file.to_string(), value);
                    }
                }
            }
        }
        Self {
            locale: locale.to_string(),
            corpus_dir: if dir_exists { Some(corpus_dir) } else { None },
            cache,
        }
    }

    /// Try to sample an accounting plan entry from the external corpus.
    /// Returns (code, local_name, english_name) if found.
    /// `level` filters by code digit length: 1 = groups, 2 = subgroups, 3+ = accounts.
    fn sample_accounting_plan(
        &self,
        country: &str,
        level: u64,
        rng: &mut dyn rand::RngCore,
    ) -> Option<(String, String, String)> {
        let arr = self.cache.get("shared/accounting_plans.json")?.as_array()?;
        let filtered: Vec<_> = arr
            .iter()
            .filter(|v| {
                let c = v.get("country").and_then(|c| c.as_str()).unwrap_or("");
                let l = v.get("level").and_then(|l| l.as_u64()).unwrap_or(0);
                c == country && l == level
            })
            .collect();
        if filtered.is_empty() {
            return None;
        }
        let entry = filtered[rng.gen_range(0..filtered.len())];
        let code = entry.get("code").and_then(|v| v.as_str())?.to_string();
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let name_en = entry
            .get("name_en")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Some((code, name, name_en))
    }

    /// Try to sample a first name from the external corpus (weighted).
    fn sample_first_name_from_corpus(&self, rng: &mut dyn rand::RngCore) -> Option<String> {
        let arr = self.cache.get("en-US/person_first.json")?.as_array()?;
        sample_weighted_name(arr, rng)
    }

    /// Try to sample a last name from the external corpus (weighted).
    fn sample_last_name_from_corpus(&self, rng: &mut dyn rand::RngCore) -> Option<String> {
        let arr = self.cache.get("en-US/person_last.json")?.as_array()?;
        sample_weighted_name(arr, rng)
    }
}

/// Sample from a JSON array using weighted selection on a `weight` field,
/// returning the value of the `name` field.
fn sample_weighted_name(arr: &[serde_json::Value], rng: &mut dyn rand::RngCore) -> Option<String> {
    if arr.is_empty() {
        return None;
    }
    let weights: Vec<f64> = arr
        .iter()
        .map(|v| v.get("weight").and_then(|w| w.as_f64()).unwrap_or(1.0))
        .collect();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = rng.gen_range(0.0..total);
    for (i, w) in weights.iter().enumerate() {
        roll -= w;
        if roll <= 0.0 {
            return arr[i]
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from);
        }
    }
    arr.last()
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Sample from a JSON array weighted by a numeric field (e.g. `population`),
/// returning the value of the given `name_field`.
fn sample_weighted_by(
    arr: &[serde_json::Value],
    weight_field: &str,
    name_field: &str,
    rng: &mut dyn rand::RngCore,
) -> Option<String> {
    if arr.is_empty() {
        return None;
    }
    let weights: Vec<f64> = arr
        .iter()
        .map(|v| v.get(weight_field).and_then(|w| w.as_f64()).unwrap_or(1.0))
        .collect();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = rng.gen_range(0.0..total);
    for (i, w) in weights.iter().enumerate() {
        roll -= w;
        if roll <= 0.0 {
            return arr[i]
                .get(name_field)
                .and_then(|v| v.as_str())
                .map(String::from);
        }
    }
    arr.last()
        .and_then(|v| v.get(name_field))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Sample uniformly from a JSON array, returning the value of the given field.
fn sample_uniform(
    arr: &[serde_json::Value],
    field: &str,
    rng: &mut dyn rand::RngCore,
) -> Option<String> {
    if arr.is_empty() {
        return None;
    }
    let idx = rng.gen_range(0..arr.len());
    arr[idx]
        .get(field)
        .and_then(|v| v.as_str())
        .map(String::from)
}

impl CorpusProvider for CorpusRegistry {
    fn sample(
        &self,
        semantic: &SemanticType,
        rng: &mut dyn rand::RngCore,
    ) -> Result<Value, DatjitError> {
        let full = semantic.full_name();
        let val = match full.as_str() {
            // ── Person ──────────────────────────────────────────────
            "person.full" => {
                let first = self
                    .sample_first_name_from_corpus(rng)
                    .unwrap_or_else(|| sample_first_name(rng).to_string());
                let last = self
                    .sample_last_name_from_corpus(rng)
                    .unwrap_or_else(|| pick(embedded::LAST_NAMES, rng).to_string());
                Value::String(format!("{first} {last}"))
            }
            "person.first" => {
                if let Some(name) = self.sample_first_name_from_corpus(rng) {
                    Value::String(name)
                } else {
                    Value::String(sample_first_name(rng).to_string())
                }
            }
            "person.last" => {
                if let Some(name) = self.sample_last_name_from_corpus(rng) {
                    Value::String(name)
                } else {
                    Value::String(pick(embedded::LAST_NAMES, rng).to_string())
                }
            }
            "person.username" => {
                let first = self
                    .sample_first_name_from_corpus(rng)
                    .unwrap_or_else(|| sample_first_name(rng).to_string())
                    .to_lowercase();
                let num = rng.gen_range(1..999);
                Value::String(format!("{first}{num}"))
            }
            "person.prefix" => Value::String(pick(embedded::PERSON_PREFIXES, rng).to_string()),
            "person.suffix" => Value::String(pick(embedded::PERSON_SUFFIXES, rng).to_string()),
            "person.gender" => Value::String(pick(embedded::GENDERS, rng).to_string()),
            "person.dob" => {
                let year = rng.gen_range(1950..=2005);
                let month = rng.gen_range(1..=12);
                let day = rng.gen_range(1..=28);
                Value::Date(format!("{year:04}-{month:02}-{day:02}"))
            }
            "person.age" => Value::Int(rng.gen_range(18..=85)),
            "person.bio" => {
                let first = self
                    .sample_first_name_from_corpus(rng)
                    .unwrap_or_else(|| sample_first_name(rng).to_string());
                let title = if let Some(arr) = self
                    .cache
                    .get("shared/job_titles.json")
                    .and_then(|v| v.as_array())
                {
                    sample_uniform(arr, "title", rng).unwrap_or_else(|| {
                        let (t, _) =
                            embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                        t.to_string()
                    })
                } else {
                    let (t, _) = embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                    t.to_string()
                };
                let (city, state) = if let Some(arr) = self
                    .cache
                    .get("en-US/cities.json")
                    .and_then(|v| v.as_array())
                {
                    let city_name = sample_weighted_by(arr, "population", "name", rng);
                    // For state in bio, fall back to embedded since city corpus may not have state
                    if let Some(c) = city_name {
                        let (_, s, _, _) = pick_city(rng);
                        (c, s.to_string())
                    } else {
                        let (c, s, _, _) = pick_city(rng);
                        (c.to_string(), s.to_string())
                    }
                } else {
                    let (c, s, _, _) = pick_city(rng);
                    (c.to_string(), s.to_string())
                };
                Value::String(format!("{first} is a {title} based in {city}, {state}."))
            }
            "person.avatar" => {
                let n = rng.gen_range(1..10000);
                Value::String(format!("https://i.pravatar.cc/150?u={n}"))
            }
            "person.ssn" => {
                let a = rng.gen_range(100..999);
                let b = rng.gen_range(10..99);
                let c = rng.gen_range(1000..9999);
                Value::String(format!("{a}-{b}-{c}"))
            }

            // ── Email ───────────────────────────────────────────────
            "email" => {
                let first = self
                    .sample_first_name_from_corpus(rng)
                    .unwrap_or_else(|| sample_first_name(rng).to_string())
                    .to_lowercase();
                let last = self
                    .sample_last_name_from_corpus(rng)
                    .unwrap_or_else(|| pick(embedded::LAST_NAMES, rng).to_string())
                    .to_lowercase();
                let domain = pick_email_domain(rng);
                Value::String(format!("{first}.{last}@{domain}"))
            }

            // ── Phone ───────────────────────────────────────────────
            "phone" => {
                let area = rng.gen_range(200..999);
                let ex = rng.gen_range(200..999);
                let num = rng.gen_range(1000..9999);
                Value::String(format!("+1-{area}-{ex}-{num}"))
            }
            "phone.mobile" | "phone.landline" => {
                let area = rng.gen_range(200..999);
                let ex = rng.gen_range(200..999);
                let num = rng.gen_range(1000..9999);
                Value::String(format!("({area}) {ex}-{num}"))
            }

            // ── URL ─────────────────────────────────────────────────
            "url" => {
                let word = pick(embedded::WORDS, rng);
                let slug = gen_slug(rng);
                Value::String(format!("https://{word}.example.com/{slug}"))
            }
            "url.image" => {
                let n = rng.gen_range(1..10000);
                Value::String(format!("https://picsum.photos/seed/{n}/800/600"))
            }
            "url.avatar" => {
                let n = rng.gen_range(1..10000);
                Value::String(format!("https://i.pravatar.cc/150?u={n}"))
            }

            // ── IP & MAC ────────────────────────────────────────────
            "ipv4" => {
                let a = rng.gen_range(1..255);
                let b = rng.gen_range(0..255);
                let c = rng.gen_range(0..255);
                let d = rng.gen_range(1..255);
                Value::String(format!("{a}.{b}.{c}.{d}"))
            }
            "ipv6" => {
                let groups: Vec<String> = (0..8)
                    .map(|_| format!("{:04x}", rng.gen_range(0..=0xFFFFu32)))
                    .collect();
                Value::String(groups.join(":"))
            }
            "mac" => {
                let octets: Vec<String> = (0..6)
                    .map(|_| format!("{:02X}", rng.gen_range(0..=0xFFu32)))
                    .collect();
                Value::String(octets.join(":"))
            }

            // ── Address ─────────────────────────────────────────────
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
                if let Some(arr) = self
                    .cache
                    .get("en-US/cities.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_weighted_by(arr, "population", "name", rng) {
                        Value::String(name)
                    } else {
                        let (city, _, _, _) = pick_city(rng);
                        Value::String(city.to_string())
                    }
                } else {
                    let (city, _, _, _) = pick_city(rng);
                    Value::String(city.to_string())
                }
            }
            "address.state" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/admin1.json")
                    .and_then(|v| v.as_array())
                {
                    let us_states: Vec<&serde_json::Value> = arr
                        .iter()
                        .filter(|v| v.get("country").and_then(|c| c.as_str()) == Some("US"))
                        .collect();
                    if let Some(entry) = us_states.get(rng.gen_range(0..us_states.len().max(1))) {
                        if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                            Value::String(name.to_string())
                        } else {
                            let (_, state, _, _) = pick_city(rng);
                            Value::String(state.to_string())
                        }
                    } else {
                        let (_, state, _, _) = pick_city(rng);
                        Value::String(state.to_string())
                    }
                } else {
                    let (_, state, _, _) = pick_city(rng);
                    Value::String(state.to_string())
                }
            }
            "address.zip" => {
                if let Some(arr) = self
                    .cache
                    .get("en-US/postal_codes.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(zip) = sample_uniform(arr, "zip", rng) {
                        Value::String(zip)
                    } else {
                        let (_, _, zip, _) = pick_city(rng);
                        Value::String(zip.to_string())
                    }
                } else {
                    let (_, _, zip, _) = pick_city(rng);
                    Value::String(zip.to_string())
                }
            }
            "address.country" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/countries.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String("US".into())
                    }
                } else {
                    Value::String("US".into())
                }
            }

            // ── Geo ─────────────────────────────────────────────────
            "geo.lat" => {
                let lat = rng.gen_range(-90.0_f64..=90.0);
                Value::Float((lat * 1_000_000.0).round() / 1_000_000.0)
            }
            "geo.lng" => {
                let lng = rng.gen_range(-180.0_f64..=180.0);
                Value::Float((lng * 1_000_000.0).round() / 1_000_000.0)
            }
            "geo.point" => {
                let lat = rng.gen_range(-90.0_f64..=90.0);
                let lng = rng.gen_range(-180.0_f64..=180.0);
                let lat_r = (lat * 1_000_000.0).round() / 1_000_000.0;
                let lng_r = (lng * 1_000_000.0).round() / 1_000_000.0;
                Value::Tuple(vec![Value::Float(lat_r), Value::Float(lng_r)])
            }

            // ── Timezone ────────────────────────────────────────────
            "timezone" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/timezones.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(tz) = sample_uniform(arr, "timezone", rng) {
                        Value::String(tz)
                    } else {
                        let (_, _, _, tz) = pick_city(rng);
                        Value::String(tz.to_string())
                    }
                } else {
                    let (_, _, _, tz) = pick_city(rng);
                    Value::String(tz.to_string())
                }
            }

            // ── Currency ────────────────────────────────────────────
            "currency.usd" | "currency.eur" => {
                let cents = rng.gen_range(1..=999999);
                Value::Float(cents as f64 / 100.0)
            }

            // ── Credit Card ─────────────────────────────────────────
            "credit_card" => Value::String(gen_credit_card(rng)),
            "credit_card.type" => {
                let (name, _) = embedded::CREDIT_CARD_TYPES
                    [rng.gen_range(0..embedded::CREDIT_CARD_TYPES.len())];
                Value::String(name.to_string())
            }

            // ── IBAN ────────────────────────────────────────────────
            "iban" => {
                let bank: String = (0..4).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
                let account: String = (0..14)
                    .map(|_| rng.gen_range(b'0'..=b'9') as char)
                    .collect();
                let check = rng.gen_range(10..99);
                Value::String(format!("GB{check}{bank}{account}"))
            }

            // ── SWIFT ───────────────────────────────────────────────
            "swift" => {
                let bank: String = (0..4).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
                let country: String = (0..2).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
                let location: String = (0..2).map(|_| rng.gen_range(b'0'..=b'9') as char).collect();
                Value::String(format!("{bank}{country}{location}"))
            }

            // ── Text ────────────────────────────────────────────────
            "text.word" => Value::String(pick(embedded::WORDS, rng).to_string()),
            "text.sentence" => Value::String(gen_sentence(embedded::WORDS, rng)),
            "text.paragraph" => Value::String(gen_paragraph(embedded::WORDS, rng)),
            "text.paragraphs" => {
                let count = semantic
                    .params
                    .first()
                    .and_then(|p| p.parse::<usize>().ok())
                    .unwrap_or(3);
                let paragraphs: Vec<String> = (0..count)
                    .map(|_| gen_paragraph(embedded::WORDS, rng))
                    .collect();
                Value::String(paragraphs.join("\n\n"))
            }
            "text.slug" => Value::String(gen_slug(rng)),
            "text.markdown" => {
                let heading_word = pick(embedded::WORDS, rng);
                let heading = format!(
                    "# {}{}",
                    heading_word[..1].to_uppercase(),
                    &heading_word[1..]
                );
                let body = gen_paragraph(embedded::WORDS, rng);
                Value::String(format!("{heading}\n\n{body}"))
            }
            "text.html" => {
                let body = gen_paragraph(embedded::WORDS, rng);
                Value::String(format!("<p>{body}</p>"))
            }
            "text.lorem" => {
                let count = semantic
                    .params
                    .first()
                    .and_then(|p| p.parse::<usize>().ok())
                    .unwrap_or(20);
                let words: Vec<&str> = (0..count)
                    .map(|_| pick(embedded::LOREM_WORDS, rng))
                    .collect();
                let mut text = words.join(" ");
                if let Some(first) = text.get_mut(..1) {
                    first.make_ascii_uppercase();
                }
                text.push('.');
                Value::String(text)
            }

            // ── Product ─────────────────────────────────────────────
            "product.title" => {
                let adj = pick(embedded::PRODUCT_ADJECTIVES, rng);
                let mat = pick(embedded::PRODUCT_MATERIALS, rng);
                let noun = pick(embedded::PRODUCT_NOUNS, rng);
                Value::String(format!("{adj} {mat} {noun}"))
            }
            "product.description" => {
                let adj = pick(embedded::PRODUCT_ADJECTIVES, rng);
                let mat = pick(embedded::PRODUCT_MATERIALS, rng);
                let noun = pick(embedded::PRODUCT_NOUNS, rng);
                Value::String(format!(
                    "This {adj} {noun} is made from high-quality {mat} for lasting durability and style."
                ).to_lowercase())
            }
            "product.sku" => Value::String(gen_sku(rng)),

            // ── Company ─────────────────────────────────────────────
            "company.name" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/companies.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        let prefix = pick(embedded::COMPANY_PREFIXES, rng);
                        let core = pick(embedded::COMPANY_CORES, rng);
                        let suffix = pick(embedded::COMPANY_SUFFIXES, rng);
                        Value::String(format!("{prefix} {core} {suffix}"))
                    }
                } else {
                    let prefix = pick(embedded::COMPANY_PREFIXES, rng);
                    let core = pick(embedded::COMPANY_CORES, rng);
                    let suffix = pick(embedded::COMPANY_SUFFIXES, rng);
                    Value::String(format!("{prefix} {core} {suffix}"))
                }
            }
            "company.industry" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/companies.json")
                    .and_then(|v| v.as_array())
                {
                    // Filter to entries with a non-empty industry field
                    let with_industry: Vec<&serde_json::Value> = arr
                        .iter()
                        .filter(|v| {
                            v.get("industry")
                                .and_then(|i| i.as_str())
                                .map(|s| !s.is_empty())
                                .unwrap_or(false)
                        })
                        .collect();
                    if !with_industry.is_empty() {
                        let idx = rng.gen_range(0..with_industry.len());
                        let industry = with_industry[idx]
                            .get("industry")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !industry.is_empty() {
                            Value::String(industry)
                        } else {
                            Value::String(pick(embedded::INDUSTRIES, rng).to_string())
                        }
                    } else {
                        Value::String(pick(embedded::INDUSTRIES, rng).to_string())
                    }
                } else {
                    Value::String(pick(embedded::INDUSTRIES, rng).to_string())
                }
            }
            "company.catch_phrase" => {
                let verb = pick(embedded::CATCH_PHRASE_VERBS, rng);
                let adj = pick(embedded::CATCH_PHRASE_ADJECTIVES, rng);
                let noun = pick(embedded::CATCH_PHRASE_NOUNS, rng);
                Value::String(format!("{verb} {adj} {noun}."))
            }

            // ── Job ─────────────────────────────────────────────────
            "job.title" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/job_titles.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(title) = sample_uniform(arr, "title", rng) {
                        Value::String(title)
                    } else {
                        let (title, _) =
                            embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                        Value::String(title.to_string())
                    }
                } else {
                    let (title, _) =
                        embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                    Value::String(title.to_string())
                }
            }
            "job.department" => {
                // External corpus has SOC codes, not departments — use embedded fallback
                let (_, dept) = embedded::JOB_TITLES[rng.gen_range(0..embedded::JOB_TITLES.len())];
                Value::String(dept.to_string())
            }

            // ── Color ───────────────────────────────────────────────
            "color.name" => {
                let (name, _) =
                    embedded::COLOR_NAMES[rng.gen_range(0..embedded::COLOR_NAMES.len())];
                Value::String(name.to_string())
            }
            "color.hex" => {
                let (_, hex) = embedded::COLOR_NAMES[rng.gen_range(0..embedded::COLOR_NAMES.len())];
                Value::String(hex.to_string())
            }
            "color.rgb" => {
                let r = rng.gen_range(0..=255);
                let g = rng.gen_range(0..=255);
                let b = rng.gen_range(0..=255);
                Value::String(format!("rgb({r}, {g}, {b})"))
            }

            // ── File ────────────────────────────────────────────────
            "file.name" => {
                let w1 = pick(embedded::WORDS, rng);
                let w2 = pick(embedded::WORDS, rng);
                let (ext, _) = embedded::FILE_TYPES[rng.gen_range(0..embedded::FILE_TYPES.len())];
                Value::String(format!("{w1}_{w2}.{ext}"))
            }
            "file.extension" => {
                let (ext, _) = embedded::FILE_TYPES[rng.gen_range(0..embedded::FILE_TYPES.len())];
                Value::String(format!(".{ext}"))
            }
            "file.mime" => {
                let (_, mime) = embedded::FILE_TYPES[rng.gen_range(0..embedded::FILE_TYPES.len())];
                Value::String(mime.to_string())
            }

            // ── Identifiers ─────────────────────────────────────────
            "sku" => Value::String(gen_sku(rng)),
            "slug" => Value::String(gen_slug(rng)),
            "code" => {
                let letters: String = (0..3).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
                let digits = rng.gen_range(100..999);
                Value::String(format!("{letters}{digits}"))
            }

            // ── Hashes ──────────────────────────────────────────────
            "hash.md5" => Value::String(gen_hex_string(32, rng)),
            "hash.sha256" => Value::String(gen_hex_string(64, rng)),

            // ── Accounting ─────────────────────────────────────────
            "accounting.country" => {
                let country = pick(embedded::ACCOUNTING_COUNTRIES, rng);
                Value::String(country.to_string())
            }
            "accounting.group" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 1, rng) {
                    Value::String(entry.0.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_GROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("1".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.1.to_string())
                    }
                }
            }
            "accounting.group_name" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 1, rng) {
                    Value::String(entry.1.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_GROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("Capital Accounts".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.2.to_string())
                    }
                }
            }
            "accounting.group_name_en" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 1, rng) {
                    Value::String(entry.2.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_GROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("Capital Accounts".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.3.to_string())
                    }
                }
            }
            "accounting.subgroup" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 2, rng) {
                    Value::String(entry.0.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_SUBGROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("10".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.1.to_string())
                    }
                }
            }
            "accounting.subgroup_name" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 2, rng) {
                    Value::String(entry.1.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_SUBGROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("Capital".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.2.to_string())
                    }
                }
            }
            "accounting.subgroup_name_en" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self.sample_accounting_plan(country, 2, rng) {
                    Value::String(entry.2.to_string())
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_SUBGROUPS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        Value::String("Capital".into())
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.3.to_string())
                    }
                }
            }

            "accounting.account" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                // Try external corpus level 3, then 4, then embedded
                if let Some(entry) = self
                    .sample_accounting_plan(country, 3, rng)
                    .or_else(|| self.sample_accounting_plan(country, 4, rng))
                {
                    Value::String(entry.0)
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_ACCOUNTS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        // Fall back to 4-digit
                        let entries4: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_4
                            .iter()
                            .filter(|(c, _, _, _)| *c == country)
                            .collect();
                        if entries4.is_empty() {
                            Value::String("100".into())
                        } else {
                            let e = entries4[rng.gen_range(0..entries4.len())];
                            Value::String(e.1.to_string())
                        }
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.1.to_string())
                    }
                }
            }
            "accounting.account_name" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self
                    .sample_accounting_plan(country, 3, rng)
                    .or_else(|| self.sample_accounting_plan(country, 4, rng))
                {
                    Value::String(entry.1)
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_ACCOUNTS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        let entries4: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_4
                            .iter()
                            .filter(|(c, _, _, _)| *c == country)
                            .collect();
                        if entries4.is_empty() {
                            Value::String("Capital social".into())
                        } else {
                            let e = entries4[rng.gen_range(0..entries4.len())];
                            Value::String(e.2.to_string())
                        }
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.2.to_string())
                    }
                }
            }
            "accounting.account_name_en" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self
                    .sample_accounting_plan(country, 3, rng)
                    .or_else(|| self.sample_accounting_plan(country, 4, rng))
                {
                    Value::String(entry.2)
                } else {
                    let entries: Vec<_> = embedded::ACCOUNTING_ACCOUNTS
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if entries.is_empty() {
                        let entries4: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_4
                            .iter()
                            .filter(|(c, _, _, _)| *c == country)
                            .collect();
                        if entries4.is_empty() {
                            Value::String("Share capital".into())
                        } else {
                            let e = entries4[rng.gen_range(0..entries4.len())];
                            Value::String(e.3.to_string())
                        }
                    } else {
                        let e = entries[rng.gen_range(0..entries.len())];
                        Value::String(e.3.to_string())
                    }
                }
            }
            "accounting.account_full" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                // Pick the deepest available level: 5 → 4 → 3
                if let Some(entry) = self
                    .sample_accounting_plan(country, 5, rng)
                    .or_else(|| self.sample_accounting_plan(country, 4, rng))
                    .or_else(|| self.sample_accounting_plan(country, 3, rng))
                {
                    Value::String(entry.0)
                } else {
                    // Try embedded: 5 → 4 → 3
                    let e5: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_5
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if !e5.is_empty() {
                        let e = e5[rng.gen_range(0..e5.len())];
                        Value::String(e.1.to_string())
                    } else {
                        let e4: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_4
                            .iter()
                            .filter(|(c, _, _, _)| *c == country)
                            .collect();
                        if !e4.is_empty() {
                            let e = e4[rng.gen_range(0..e4.len())];
                            Value::String(e.1.to_string())
                        } else {
                            let e3: Vec<_> = embedded::ACCOUNTING_ACCOUNTS
                                .iter()
                                .filter(|(c, _, _, _)| *c == country)
                                .collect();
                            if e3.is_empty() {
                                Value::String("1000".into())
                            } else {
                                let e = e3[rng.gen_range(0..e3.len())];
                                Value::String(e.1.to_string())
                            }
                        }
                    }
                }
            }
            "accounting.account_full_name" => {
                let country = semantic
                    .params
                    .first()
                    .map(|s| strip_quotes(s.as_str()))
                    .unwrap_or_else(|| pick(embedded::ACCOUNTING_COUNTRIES, rng));
                if let Some(entry) = self
                    .sample_accounting_plan(country, 5, rng)
                    .or_else(|| self.sample_accounting_plan(country, 4, rng))
                    .or_else(|| self.sample_accounting_plan(country, 3, rng))
                {
                    Value::String(entry.1)
                } else {
                    let e5: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_5
                        .iter()
                        .filter(|(c, _, _, _)| *c == country)
                        .collect();
                    if !e5.is_empty() {
                        let e = e5[rng.gen_range(0..e5.len())];
                        Value::String(e.2.to_string())
                    } else {
                        let e4: Vec<_> = embedded::ACCOUNTING_ACCOUNTS_4
                            .iter()
                            .filter(|(c, _, _, _)| *c == country)
                            .collect();
                        if !e4.is_empty() {
                            let e = e4[rng.gen_range(0..e4.len())];
                            Value::String(e.2.to_string())
                        } else {
                            let e3: Vec<_> = embedded::ACCOUNTING_ACCOUNTS
                                .iter()
                                .filter(|(c, _, _, _)| *c == country)
                                .collect();
                            if e3.is_empty() {
                                Value::String("Capital social".into())
                            } else {
                                let e = e3[rng.gen_range(0..e3.len())];
                                Value::String(e.2.to_string())
                            }
                        }
                    }
                }
            }

            // ── ERP domain types (Odoo corpus) ──────────────────────
            "erp.payment_term" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/erp_payment_terms.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String("Net 30".into())
                    }
                } else {
                    Value::String(pick(embedded::ERP_PAYMENT_TERMS, rng).to_string())
                }
            }
            "erp.incoterm" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/erp_incoterms.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(code) = sample_uniform(arr, "code", rng) {
                        Value::String(code)
                    } else {
                        Value::String("FOB".into())
                    }
                } else {
                    Value::String(pick(embedded::ERP_INCOTERMS, rng).to_string())
                }
            }
            "erp.uom" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/erp_uom.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String("Unit(s)".into())
                    }
                } else {
                    Value::String(pick(embedded::ERP_UOM, rng).to_string())
                }
            }
            "erp.tax_rate" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/erp_tax_rates.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String("VAT 21%".into())
                    }
                } else {
                    Value::String(pick(embedded::ERP_TAX_RATES, rng).to_string())
                }
            }
            "erp.account_type" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/erp_account_types.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String("Asset Receivable".into())
                    }
                } else {
                    Value::String(pick(embedded::ERP_ACCOUNT_TYPES, rng).to_string())
                }
            }

            // ── Ecommerce ──────────────────────────────────────────
            "ecommerce.order_status" => {
                Value::String(pick(embedded::ORDER_STATUSES, rng).to_string())
            }
            "ecommerce.payment_method" => {
                Value::String(pick(embedded::PAYMENT_METHODS, rng).to_string())
            }
            "ecommerce.shipping_carrier" => {
                let idx = rng.gen_range(0..embedded::SHIPPING_CARRIERS.len());
                let (name, _) = embedded::SHIPPING_CARRIERS[idx];
                Value::String(name.to_string())
            }
            "ecommerce.tracking_number" => {
                let idx = rng.gen_range(0..embedded::SHIPPING_CARRIERS.len());
                let (_, prefix) = embedded::SHIPPING_CARRIERS[idx];
                Value::String(format!(
                    "{}{:012}",
                    prefix,
                    rng.gen_range(100_000_000_000u64..999_999_999_999u64)
                ))
            }
            "ecommerce.return_reason" => {
                Value::String(pick(embedded::RETURN_REASONS, rng).to_string())
            }
            "ecommerce.discount_type" => {
                Value::String(pick(embedded::DISCOUNT_TYPES, rng).to_string())
            }
            "ecommerce.fulfillment_status" => {
                Value::String(pick(embedded::FULFILLMENT_STATUSES, rng).to_string())
            }
            "ecommerce.department" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/instacart_departments.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String(pick(embedded::ECOMMERCE_DEPARTMENTS, rng).to_string())
                    }
                } else {
                    Value::String(pick(embedded::ECOMMERCE_DEPARTMENTS, rng).to_string())
                }
            }
            "ecommerce.aisle" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/instacart_aisles.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(name) = sample_uniform(arr, "name", rng) {
                        Value::String(name)
                    } else {
                        Value::String(pick(embedded::ECOMMERCE_AISLES, rng).to_string())
                    }
                } else {
                    Value::String(pick(embedded::ECOMMERCE_AISLES, rng).to_string())
                }
            }
            "ecommerce.product_category" => {
                if let Some(arr) = self
                    .cache
                    .get("shared/instacart_products.json")
                    .and_then(|v| v.as_array())
                {
                    if let Some(dept) = sample_uniform(arr, "department", rng) {
                        Value::String(dept)
                    } else {
                        Value::String("General Merchandise".into())
                    }
                } else {
                    Value::String(pick(embedded::ECOMMERCE_DEPARTMENTS, rng).to_string())
                }
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

/// Strip surrounding quotes from a string parameter (e.g. `"ES"` → `ES`).
fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn pick_city(
    rng: &mut dyn rand::RngCore,
) -> &'static (&'static str, &'static str, &'static str, &'static str) {
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

fn gen_slug(rng: &mut dyn rand::RngCore) -> String {
    let count = rng.gen_range(2..=4);
    let words: Vec<&str> = (0..count).map(|_| pick(embedded::WORDS, rng)).collect();
    words.join("-")
}

fn gen_sentence(words: &[&str], rng: &mut dyn rand::RngCore) -> String {
    let count = rng.gen_range(5..=12);
    let selected: Vec<&str> = (0..count)
        .map(|_| words[rng.gen_range(0..words.len())])
        .collect();
    let mut s = selected.join(" ");
    if let Some(first) = s.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    s.push('.');
    s
}

fn gen_paragraph(words: &[&str], rng: &mut dyn rand::RngCore) -> String {
    let count = rng.gen_range(3..=6);
    let sentences: Vec<String> = (0..count).map(|_| gen_sentence(words, rng)).collect();
    sentences.join(" ")
}

fn gen_sku(rng: &mut dyn rand::RngCore) -> String {
    let letters: String = (0..2).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect();
    let digits = rng.gen_range(1000..9999);
    format!("SKU-{letters}-{digits}")
}

fn gen_hex_string(len: usize, rng: &mut dyn rand::RngCore) -> String {
    (0..len)
        .map(|_| format!("{:x}", rng.gen_range(0..16u8)))
        .collect()
}

fn gen_credit_card(rng: &mut dyn rand::RngCore) -> String {
    let (_, prefix) =
        embedded::CREDIT_CARD_TYPES[rng.gen_range(0..embedded::CREDIT_CARD_TYPES.len())];
    // Generate 15 digits (prefix + 14 random), then compute Luhn check digit
    let mut digits: Vec<u8> = vec![prefix.as_bytes()[0] - b'0'];
    for _ in 1..15 {
        digits.push(rng.gen_range(0..10));
    }
    // Luhn check digit
    let check = luhn_check_digit(&digits);
    digits.push(check);
    let s: String = digits.iter().map(|d| (b'0' + d) as char).collect();
    format!("{}-{}-{}-{}", &s[0..4], &s[4..8], &s[8..12], &s[12..16])
}

fn luhn_check_digit(digits: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for (i, &d) in digits.iter().rev().enumerate() {
        let mut val = d as u32;
        if i % 2 == 0 {
            val *= 2;
            if val > 9 {
                val -= 9;
            }
        }
        sum += val;
    }
    ((10 - (sum % 10)) % 10) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    fn sample_type(ns: &str, tag: &str) -> Value {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new(ns, tag);
        registry.sample(&st, &mut rng()).unwrap()
    }

    fn sample_type_str(ns: &str, tag: &str) -> String {
        match sample_type(ns, tag) {
            Value::String(s) => s,
            Value::Date(s) => s,
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn test_person_full() {
        let s = sample_type_str("person", "full");
        assert!(s.contains(' '), "should have first and last name");
    }

    #[test]
    fn test_person_prefix() {
        let s = sample_type_str("person", "prefix");
        assert!(s.ends_with('.'), "prefix should end with period");
    }

    #[test]
    fn test_person_dob() {
        let s = sample_type_str("person", "dob");
        assert_eq!(s.len(), 10, "dob should be YYYY-MM-DD");
        assert!(s.starts_with("19") || s.starts_with("20"));
    }

    #[test]
    fn test_person_age() {
        match sample_type("person", "age") {
            Value::Int(n) => assert!((18..=85).contains(&n)),
            other => panic!("expected Int, got {other:?}"),
        }
    }

    #[test]
    fn test_person_ssn() {
        let s = sample_type_str("person", "ssn");
        assert_eq!(s.len(), 11, "SSN should be NNN-NN-NNNN");
        assert_eq!(s.matches('-').count(), 2);
    }

    #[test]
    fn test_email() {
        let s = sample_type_str("email", "");
        assert!(s.contains('@'));
    }

    #[test]
    fn test_phone() {
        let s = sample_type_str("phone", "");
        assert!(s.starts_with("+1-"));
    }

    #[test]
    fn test_phone_mobile() {
        let s = sample_type_str("phone", "mobile");
        assert!(s.starts_with('('));
    }

    #[test]
    fn test_url() {
        let s = sample_type_str("url", "");
        assert!(s.starts_with("https://"));
    }

    #[test]
    fn test_ipv4() {
        let s = sample_type_str("ipv4", "");
        assert_eq!(s.split('.').count(), 4);
    }

    #[test]
    fn test_ipv6() {
        let s = sample_type_str("ipv6", "");
        assert_eq!(s.split(':').count(), 8);
    }

    #[test]
    fn test_mac() {
        let s = sample_type_str("mac", "");
        assert_eq!(s.split(':').count(), 6);
    }

    #[test]
    fn test_geo_lat() {
        match sample_type("geo", "lat") {
            Value::Float(n) => assert!((-90.0..=90.0).contains(&n)),
            other => panic!("expected Float, got {other:?}"),
        }
    }

    #[test]
    fn test_geo_point() {
        match sample_type("geo", "point") {
            Value::Tuple(items) => assert_eq!(items.len(), 2),
            other => panic!("expected Tuple, got {other:?}"),
        }
    }

    #[test]
    fn test_credit_card() {
        let s = sample_type_str("credit_card", "");
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        assert_eq!(digits.len(), 16);
        // Verify Luhn
        let d: Vec<u8> = digits.bytes().map(|b| b - b'0').collect();
        assert!(luhn_validate(&d));
    }

    #[test]
    fn test_iban() {
        let s = sample_type_str("iban", "");
        assert!(s.starts_with("GB"));
        assert_eq!(s.len(), 22);
    }

    #[test]
    fn test_swift() {
        let s = sample_type_str("swift", "");
        assert_eq!(s.len(), 8);
    }

    #[test]
    fn test_text_word() {
        let s = sample_type_str("text", "word");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_text_sentence() {
        let s = sample_type_str("text", "sentence");
        assert!(s.ends_with('.'));
        assert!(s.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_text_slug() {
        let s = sample_type_str("text", "slug");
        assert!(s.contains('-'));
        assert!(!s.contains(' '));
    }

    #[test]
    fn test_text_html() {
        let s = sample_type_str("text", "html");
        assert!(s.starts_with("<p>"));
        assert!(s.ends_with("</p>"));
    }

    #[test]
    fn test_product_title() {
        let s = sample_type_str("product", "title");
        assert!(s.split_whitespace().count() >= 3);
    }

    #[test]
    fn test_product_sku() {
        let s = sample_type_str("product", "sku");
        assert!(s.starts_with("SKU-"));
    }

    #[test]
    fn test_company_name() {
        let s = sample_type_str("company", "name");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_company_industry() {
        let s = sample_type_str("company", "industry");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_color_name() {
        let s = sample_type_str("color", "name");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_color_hex() {
        let s = sample_type_str("color", "hex");
        assert!(s.starts_with('#'));
    }

    #[test]
    fn test_color_rgb() {
        let s = sample_type_str("color", "rgb");
        assert!(s.starts_with("rgb("));
    }

    #[test]
    fn test_file_name() {
        let s = sample_type_str("file", "name");
        assert!(s.contains('.'));
    }

    #[test]
    fn test_file_extension() {
        let s = sample_type_str("file", "extension");
        assert!(s.starts_with('.'));
    }

    #[test]
    fn test_file_mime() {
        let s = sample_type_str("file", "mime");
        assert!(s.contains('/'));
    }

    #[test]
    fn test_sku() {
        let s = sample_type_str("sku", "");
        assert!(s.starts_with("SKU-"));
    }

    #[test]
    fn test_slug() {
        let s = sample_type_str("slug", "");
        assert!(s.contains('-'));
    }

    #[test]
    fn test_code() {
        let s = sample_type_str("code", "");
        assert_eq!(s.len(), 6);
    }

    #[test]
    fn test_hash_md5() {
        let s = sample_type_str("hash", "md5");
        assert_eq!(s.len(), 32);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_sha256() {
        let s = sample_type_str("hash", "sha256");
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_unknown_semantic() {
        let registry = CorpusRegistry::new("en-US");
        let st = SemanticType::new("unknown", "type");
        assert!(registry.sample(&st, &mut rng()).is_err());
    }

    fn luhn_validate(digits: &[u8]) -> bool {
        let mut sum: u32 = 0;
        for (i, &d) in digits.iter().rev().enumerate() {
            let mut val = d as u32;
            if i % 2 == 1 {
                val *= 2;
                if val > 9 {
                    val -= 9;
                }
            }
            sum += val;
        }
        sum.is_multiple_of(10)
    }
}
