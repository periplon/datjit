//! Batch 4: GitHub-hosted open data corpus sources.
//!
//! This module provides corpus sources from GitHub repositories with
//! permissive licenses (CC0, MIT, Public Domain, CC BY).

use std::fs;
use std::path::Path;

use datjit_core::error::DatjitError;
use serde::{Deserialize, Serialize};

use crate::updater::{download, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalNameEntry {
    pub name: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorEntry {
    pub name: String,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleEntry {
    pub make: String,
    pub model: String,
    pub year: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEntry {
    pub title: String,
    pub authors: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoEntry {
    pub symbol: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyDesignatorEntry {
    pub designator: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesEntry {
    pub common_name: String,
    pub scientific_name: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetNameEntry {
    pub name: String,
    pub country: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

/// Return the GitHub-hosted corpus sources (Batch 4).
pub fn github_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "International Names".into(),
            description: "Popular first/last names from 106 countries".into(),
            url: "https://github.com/sigpwned/popular-names-by-country-dataset".into(),
            license: "CC0".into(),
            category: "person".into(),
        },
        CorpusSource {
            name: "CSS Color Names".into(),
            description: "CSS named colors with hex values".into(),
            url: "https://github.com/bahamas10/css-color-names".into(),
            license: "MIT".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "US Car Models".into(),
            description: "15K+ US car models 1992-2023".into(),
            url: "https://github.com/abhionlyone/us-car-models-data".into(),
            license: "MIT".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Goodbooks 10K".into(),
            description: "10K books with titles, authors, ratings".into(),
            url: "https://github.com/zygmuntz/goodbooks-10k".into(),
            license: "CC BY-SA 4.0".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Cryptocurrencies".into(),
            description: "12K+ cryptocurrency symbols and names".into(),
            url: "https://github.com/crypti/cryptocurrencies".into(),
            license: "MIT".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Company Designators".into(),
            description: "International corporate entity designators".into(),
            url: "https://github.com/ProfoundNetworks/company_designator".into(),
            license: "CC BY-SA 3.0".into(),
            category: "company".into(),
        },
        CorpusSource {
            name: "Species Names".into(),
            description: "Scientific and common species names".into(),
            url: "https://github.com/species-names/dataset".into(),
            license: "CC0".into(),
            category: "shared".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Download all GitHub corpus sources and write results into the temp directories.
pub fn download_github_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "International Names",
        "shared/international_names.json",
        || download_and_process_international_names(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "CSS Color Names",
        "shared/color_names.json",
        || download_and_process_css_colors(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "US Car Models",
        "shared/vehicles.json",
        || download_and_process_car_models(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Goodbooks 10K",
        "shared/books.json",
        || download_and_process_books(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Cryptocurrencies",
        "shared/cryptocurrencies.json",
        || download_and_process_cryptocurrencies(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Company Designators",
        "shared/company_designators.json",
        || download_and_process_company_designators(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Species Names",
        "shared/species.json",
        || download_and_process_species(client, temp_shared),
        report,
        on_progress,
    );
}

fn download_source(
    name: &str,
    file_key: &str,
    fetch: impl FnOnce() -> Result<u64, DatjitError>,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    on_progress(&format!("Downloading {name}..."));
    match fetch() {
        Ok(size) => {
            report.files_updated.push(file_key.into());
            report.total_size_bytes += size;
            on_progress(&format!("  {file_key} ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push((name.into(), msg));
        }
    }
}

// ---------------------------------------------------------------------------
// 1. International Names (sigpwned/popular-names-by-country-dataset)
// ---------------------------------------------------------------------------

fn download_and_process_international_names(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Download forenames CSV
    let forenames_data = download(
        client,
        "https://raw.githubusercontent.com/sigpwned/popular-names-by-country-dataset/main/forenames.csv",
    )?;
    let forenames_text = String::from_utf8_lossy(&forenames_data);

    let mut entries: Vec<InternationalNameEntry> = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(forenames_text.as_bytes());

    // Expected columns: country, name, gender (or similar)
    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read intl names headers: {e}")))?
        .clone();
    let name_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("name"))
        .unwrap_or(1);
    let country_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("country"))
        .unwrap_or(0);
    let gender_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("gender"));

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        let country = record.get(country_idx).unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        let gender = gender_idx
            .and_then(|idx| record.get(idx))
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty());
        entries.push(InternationalNameEntry {
            name,
            country,
            gender,
        });
    }

    // Also download surnames CSV
    let surnames_data = download(
        client,
        "https://raw.githubusercontent.com/sigpwned/popular-names-by-country-dataset/main/surnames.csv",
    )?;
    let surnames_text = String::from_utf8_lossy(&surnames_data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(surnames_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read intl surnames headers: {e}")))?
        .clone();
    let name_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("name"))
        .unwrap_or(1);
    let country_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("country"))
        .unwrap_or(0);

    // Store surnames separately so they can be distinguished
    let mut surnames: Vec<InternationalNameEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        let country = record.get(country_idx).unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        surnames.push(InternationalNameEntry {
            name,
            country,
            gender: None,
        });
    }

    // Write both to a combined structure
    let combined = serde_json::json!({
        "forenames": entries,
        "surnames": surnames,
    });

    let json = serde_json::to_string_pretty(&combined)
        .map_err(|e| DatjitError::Corpus(format!("serialize intl names: {e}")))?;
    let path = dest_dir.join("international_names.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write intl names: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 2. CSS Color Names (bahamas10/css-color-names)
// ---------------------------------------------------------------------------

fn download_and_process_css_colors(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/bahamas10/css-color-names/master/css-color-names.json",
    )?;

    let map: std::collections::HashMap<String, String> = serde_json::from_slice(&data)
        .map_err(|e| DatjitError::Corpus(format!("parse CSS colors JSON: {e}")))?;

    let mut entries: Vec<ColorEntry> = map
        .into_iter()
        .map(|(name, hex)| ColorEntry { name, hex })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize CSS colors: {e}")))?;
    let path = dest_dir.join("color_names.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write CSS colors: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 3. US Car Models (abhionlyone/us-car-models-data)
// ---------------------------------------------------------------------------

fn download_and_process_car_models(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // This repo has CSV files per year. Download the consolidated one.
    let data = download(
        client,
        "https://raw.githubusercontent.com/abhionlyone/us-car-models-data/master/cars_data.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read car models headers: {e}")))?
        .clone();
    let make_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("make"))
        .unwrap_or(0);
    let model_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("model"))
        .unwrap_or(1);
    let year_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("year"))
        .unwrap_or(2);
    let cat_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("category") || h.to_lowercase().contains("type"));

    // Deduplicate by (make, model) keeping the latest year
    let mut seen: std::collections::HashMap<(String, String), VehicleEntry> =
        std::collections::HashMap::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let make = record.get(make_idx).unwrap_or("").trim().to_string();
        let model = record.get(model_idx).unwrap_or("").trim().to_string();
        let year = record.get(year_idx).unwrap_or("").trim().to_string();
        let category = cat_idx
            .and_then(|idx| record.get(idx))
            .unwrap_or("")
            .trim()
            .to_string();

        if make.is_empty() || model.is_empty() {
            continue;
        }

        let key = (make.clone(), model.clone());
        seen.entry(key).or_insert(VehicleEntry {
            make,
            model,
            year,
            category,
        });
    }

    let mut entries: Vec<VehicleEntry> = seen.into_values().collect();
    entries.sort_by(|a, b| a.make.cmp(&b.make).then(a.model.cmp(&b.model)));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize car models: {e}")))?;
    let path = dest_dir.join("vehicles.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write car models: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 4. Goodbooks 10K (zygmuntz/goodbooks-10k)
// ---------------------------------------------------------------------------

fn download_and_process_books(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/zygmuntz/goodbooks-10k/master/books.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<BookEntry> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read books headers: {e}")))?
        .clone();
    let title_idx = headers
        .iter()
        .position(|h| h == "title" || h == "original_title")
        .unwrap_or(0);
    let authors_idx = headers.iter().position(|h| h == "authors").unwrap_or(1);
    let lang_idx = headers.iter().position(|h| h == "language_code");

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let title = record.get(title_idx).unwrap_or("").trim().to_string();
        let authors = record.get(authors_idx).unwrap_or("").trim().to_string();
        if title.is_empty() {
            continue;
        }
        let language = lang_idx
            .and_then(|idx| record.get(idx))
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty());
        entries.push(BookEntry {
            title,
            authors,
            language,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize books: {e}")))?;
    let path = dest_dir.join("books.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write books: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 5. Cryptocurrencies (crypti/cryptocurrencies)
// ---------------------------------------------------------------------------

fn download_and_process_cryptocurrencies(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/crypti/cryptocurrencies/master/cryptocurrencies.json",
    )?;

    // The file is a JSON object: { "SYMBOL": "Name", ... }
    let map: std::collections::HashMap<String, String> = serde_json::from_slice(&data)
        .map_err(|e| DatjitError::Corpus(format!("parse crypto JSON: {e}")))?;

    let mut entries: Vec<CryptoEntry> = map
        .into_iter()
        .map(|(symbol, name)| CryptoEntry { symbol, name })
        .collect();
    entries.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize crypto: {e}")))?;
    let path = dest_dir.join("cryptocurrencies.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write crypto: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 6. Company Designators (ProfoundNetworks/company_designator)
// ---------------------------------------------------------------------------

fn download_and_process_company_designators(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/ProfoundNetworks/company_designator/master/company_designator/data/company_designator.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<CompanyDesignatorEntry> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read company designators headers: {e}")))?
        .clone();
    let designator_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("designator")
                || h.to_lowercase().contains("abbrev")
                || h.to_lowercase() == "name"
        })
        .unwrap_or(0);
    let country_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("country") || h.to_lowercase().contains("lang"))
        .unwrap_or(1);

    let mut seen = std::collections::HashSet::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let designator = record.get(designator_idx).unwrap_or("").trim().to_string();
        let country = record.get(country_idx).unwrap_or("").trim().to_string();
        if designator.is_empty() || !seen.insert(designator.clone()) {
            continue;
        }
        entries.push(CompanyDesignatorEntry {
            designator,
            country,
        });
    }

    entries.sort_by(|a, b| a.designator.cmp(&b.designator));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize company designators: {e}")))?;
    let path = dest_dir.join("company_designators.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write company designators: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 7. Species Names (species-names/dataset)
// ---------------------------------------------------------------------------

fn download_and_process_species(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // The dataset has JSON files per genus. Download the summary/index.
    // Try the main birds, mammals, and reptiles lists.
    let mut entries: Vec<SpeciesEntry> = Vec::new();

    let groups = [
        (
            "birds",
            "https://raw.githubusercontent.com/species-names/dataset/main/birds/index.json",
        ),
        (
            "mammals",
            "https://raw.githubusercontent.com/species-names/dataset/main/mammals/index.json",
        ),
        (
            "reptiles",
            "https://raw.githubusercontent.com/species-names/dataset/main/reptiles/index.json",
        ),
    ];

    for (group, url) in &groups {
        match download(client, url) {
            Ok(data) => {
                // The index files contain arrays of species objects
                if let Ok(species) = serde_json::from_slice::<Vec<serde_json::Value>>(&data) {
                    for sp in species {
                        let common_name = sp
                            .get("common_name")
                            .or_else(|| sp.get("commonName"))
                            .or_else(|| sp.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        let scientific_name = sp
                            .get("scientific_name")
                            .or_else(|| sp.get("scientificName"))
                            .or_else(|| sp.get("species"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if common_name.is_empty() && scientific_name.is_empty() {
                            continue;
                        }
                        entries.push(SpeciesEntry {
                            common_name,
                            scientific_name,
                            group: group.to_string(),
                        });
                    }
                }
                // Also try parsing as an object with genus keys
                else if let Ok(genera) = serde_json::from_slice::<
                    std::collections::HashMap<String, Vec<serde_json::Value>>,
                >(&data)
                {
                    for (_, species_list) in genera {
                        for sp in species_list {
                            let common_name = sp
                                .get("common_name")
                                .or_else(|| sp.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            let scientific_name = sp
                                .get("scientific_name")
                                .or_else(|| sp.get("species"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if common_name.is_empty() && scientific_name.is_empty() {
                                continue;
                            }
                            entries.push(SpeciesEntry {
                                common_name,
                                scientific_name,
                                group: group.to_string(),
                            });
                        }
                    }
                }
            }
            Err(_) => continue, // Skip groups that fail
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no species entries found from any group".into(),
        ));
    }

    entries.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then(a.common_name.cmp(&b.common_name))
    });

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize species: {e}")))?;
    let path = dest_dir.join("species.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write species: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_known_sources_count() {
        let sources = github_known_sources();
        assert_eq!(sources.len(), 7);
    }

    #[test]
    fn test_github_known_sources_categories() {
        let sources = github_known_sources();
        let categories: Vec<&str> = sources.iter().map(|s| s.category.as_str()).collect();
        assert!(categories.contains(&"person"));
        assert!(categories.contains(&"shared"));
        assert!(categories.contains(&"company"));
    }

    #[test]
    fn test_github_known_sources_names() {
        let sources = github_known_sources();
        let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"International Names"));
        assert!(names.contains(&"CSS Color Names"));
        assert!(names.contains(&"US Car Models"));
        assert!(names.contains(&"Goodbooks 10K"));
        assert!(names.contains(&"Cryptocurrencies"));
        assert!(names.contains(&"Company Designators"));
        assert!(names.contains(&"Species Names"));
    }

    #[test]
    fn test_github_known_sources_licenses() {
        let sources = github_known_sources();
        for source in &sources {
            assert!(
                !source.license.is_empty(),
                "source {} has no license",
                source.name
            );
        }
    }
}
