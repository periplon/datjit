//! Batch 10: Product catalog datasets.
//!
//! Downloads product data from public domain / open license sources:
//! - Open Beauty Facts (ODbL): cosmetics and beauty products
//! - Open Products Facts (ODbL): non-food consumer products
//! - PubChem (Public Domain): chemical compound names

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use datjit_core::error::DatjitError;
use serde::{Deserialize, Serialize};

use crate::updater::{download_source, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeautyProductEntry {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub brand: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerProductEntry {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub brand: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubchemCompoundEntry {
    pub cid: u64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

pub fn catalogs_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "Open Beauty Facts".into(),
            description: "Cosmetics and beauty products from Open Beauty Facts".into(),
            url: "https://static.openbeautyfacts.org/data/en.openbeautyfacts.org.products.csv.gz"
                .into(),
            license: "ODbL".into(),
            category: "ecommerce".into(),
        },
        CorpusSource {
            name: "Open Products Facts".into(),
            description: "Non-food consumer products from Open Products Facts".into(),
            url:
                "https://static.openproductsfacts.org/data/en.openproductsfacts.org.products.csv.gz"
                    .into(),
            license: "ODbL".into(),
            category: "ecommerce".into(),
        },
        CorpusSource {
            name: "PubChem Compounds".into(),
            description: "Chemical compound names from NIH PubChem".into(),
            url: "https://ftp.ncbi.nlm.nih.gov/pubchem/Compound/Extras/CID-Title.gz".into(),
            license: "Public Domain".into(),
            category: "science".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub fn download_catalogs_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "Open Beauty Facts",
        "shared/beauty_products.json",
        || download_and_process_beauty_facts(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Open Products Facts",
        "shared/consumer_products.json",
        || download_and_process_products_facts(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "PubChem Compounds",
        "shared/pubchem_compounds.json",
        || download_and_process_pubchem(client, temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// Open Beauty Facts (streaming TSV.gz, same format as Open Food Facts)
// ---------------------------------------------------------------------------

fn download_and_process_beauty_facts(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let entries = download_open_x_facts(
        client,
        "https://static.openbeautyfacts.org/data/en.openbeautyfacts.org.products.csv.gz",
        "Open Beauty Facts",
        20_000,
    )?;

    let beauty_entries: Vec<BeautyProductEntry> = entries
        .into_iter()
        .map(|(name, brand, category)| BeautyProductEntry {
            name,
            brand,
            category,
        })
        .collect();

    let json = serde_json::to_string_pretty(&beauty_entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("beauty_products.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write beauty_products.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Open Products Facts (streaming TSV.gz, same format as Open Food Facts)
// ---------------------------------------------------------------------------

fn download_and_process_products_facts(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let entries = download_open_x_facts(
        client,
        "https://static.openproductsfacts.org/data/en.openproductsfacts.org.products.csv.gz",
        "Open Products Facts",
        20_000,
    )?;

    let consumer_entries: Vec<ConsumerProductEntry> = entries
        .into_iter()
        .map(|(name, brand, category)| ConsumerProductEntry {
            name,
            brand,
            category,
        })
        .collect();

    let json = serde_json::to_string_pretty(&consumer_entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("consumer_products.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write consumer_products.json: {e}")))?;

    Ok(json.len() as u64)
}

/// Shared helper for Open *-* Facts datasets (all use the same TSV.gz format).
/// Returns Vec of (name, brand, category) tuples.
fn download_open_x_facts(
    client: &reqwest::blocking::Client,
    url: &str,
    source_name: &str,
    max_entries: usize,
) -> Result<Vec<(String, String, String)>, DatjitError> {
    let resp = client
        .get(url)
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download {source_name}: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for {source_name}",
            resp.status()
        )));
    }

    let decoder = flate2::read::GzDecoder::new(resp);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b'\t')
        .from_reader(decoder);

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("{source_name} headers: {e}")))?
        .clone();

    let name_idx = headers.iter().position(|h| h == "product_name");
    let brand_idx = headers.iter().position(|h| h == "brands");
    let cat_idx = headers
        .iter()
        .position(|h| h == "categories_en")
        .or_else(|| headers.iter().position(|h| h == "categories"));

    let name_idx = name_idx.ok_or_else(|| {
        DatjitError::Corpus(format!("{source_name}: missing product_name column"))
    })?;

    let mut entries = Vec::new();

    for result in rdr.records() {
        if entries.len() >= max_entries {
            break;
        }
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        if name.is_empty() || name.len() < 3 {
            continue;
        }
        if !name.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        let brand = brand_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        let category: String = cat_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .chars()
            .take(100)
            .collect();

        entries.push((name, brand, category));
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// PubChem CID-Title (gzipped two-column tab-separated text)
// ---------------------------------------------------------------------------

fn download_and_process_pubchem(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://ftp.ncbi.nlm.nih.gov/pubchem/Compound/Extras/CID-Title.gz")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download PubChem: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for PubChem",
            resp.status()
        )));
    }

    let decoder = flate2::read::GzDecoder::new(resp);
    let reader = BufReader::new(decoder);

    let mut entries: Vec<PubchemCompoundEntry> = Vec::new();
    let max_entries = 50_000;

    for line in reader.lines() {
        if entries.len() >= max_entries {
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let mut parts = line.splitn(2, '\t');
        let cid: u64 = match parts.next().and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => continue,
        };
        let name = match parts.next() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };

        if name.is_empty() || name.len() < 2 {
            continue;
        }

        entries.push(PubchemCompoundEntry { cid, name });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("pubchem_compounds.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write pubchem_compounds.json: {e}")))?;

    Ok(json.len() as u64)
}
