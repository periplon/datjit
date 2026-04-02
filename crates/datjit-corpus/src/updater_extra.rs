//! Batch 3: Domain-specific corpus download sources.
//!
//! This module provides 9 additional corpus sources (sources 8-16) that download
//! domain-specific reference data: foods, institutions, MAC vendors, currencies,
//! stock tickers, ICD-10 codes, locale formats, products, and companies.

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use datjit_core::error::DatjitError;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::updater::{download, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEntry {
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionEntry {
    pub name: String,
    pub city: String,
    pub state: String,
    pub control: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacVendorEntry {
    pub prefix: String,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iso4217Entry {
    pub code: String,
    pub name: String,
    pub numeric_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor_units: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTickerEntry {
    pub ticker: String,
    pub name: String,
    pub cik: u64,
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icd10Entry {
    pub code: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleFormatEntry {
    pub territory: String,
    pub first_day_of_week: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductEntry {
    pub name: String,
    pub sku: String,
    pub price: Option<f64>,
    pub manufacturer: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyEntry {
    pub name: String,
    pub country: String,
    pub industry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodProductEntry {
    pub name: String,
    pub brand: String,
    pub category: String,
    pub country: String,
    pub barcode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GermanCompanyEntry {
    pub name: String,
    pub city: String,
    pub state: String,
    pub company_type: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

/// Return the 9 extra corpus sources (Batch 3: domain-specific).
pub fn extra_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "USDA Foods".into(),
            description: "USDA FoodData Central foundation food descriptions".into(),
            url: "https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_foundation_food_csv_2024-10-31.zip".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "IPEDS Institutions".into(),
            description: "NCES IPEDS higher-education institution directory".into(),
            url: "https://nces.ed.gov/ipeds/datacenter/data/HD2023.zip".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "IEEE OUI MAC Prefixes".into(),
            description: "MAC address vendor prefix database".into(),
            url: "https://maclookup.app/downloads/csv-database/get-db".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "ISO 4217 Currencies".into(),
            description: "ISO 4217 currency codes from SIX Group".into(),
            url: "https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "SEC Stock Tickers".into(),
            description: "SEC EDGAR company ticker list".into(),
            url: "https://www.sec.gov/files/company_tickers_exchange.json".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "ICD-10 Codes".into(),
            description: "CMS ICD-10 diagnosis code descriptions".into(),
            url: "https://www.cms.gov/files/zip/2025-code-descriptions-tabular-order.zip".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "CLDR Locale Formats".into(),
            description: "Unicode CLDR week-data (first day of week per territory)".into(),
            url: "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/weekData.json".into(),
            license: "Unicode License".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Best Buy Open Products".into(),
            description: "Best Buy open product catalog data".into(),
            url: "https://raw.githubusercontent.com/BestBuyAPIs/open-data-set/master/products.json".into(),
            license: "Public Domain".into(),
            category: "product".into(),
        },
        CorpusSource {
            name: "Wikidata Companies".into(),
            description: "Company names, countries, and industries from Wikidata".into(),
            url: "https://query.wikidata.org/sparql".into(),
            license: "CC0".into(),
            category: "company".into(),
        },
        CorpusSource {
            name: "Open Food Facts".into(),
            description: "3M+ food products worldwide with name, brand, category, barcode".into(),
            url: "https://static.openfoodfacts.org/data/en.openfoodfacts.org.products.csv.gz".into(),
            license: "ODbL".into(),
            category: "food".into(),
        },
        CorpusSource {
            name: "German Handelsregister".into(),
            description: "German company register with name, city, type, status".into(),
            url: "https://daten.offeneregister.de/de_companies_ocdata.jsonl.bz2".into(),
            license: "CC0".into(),
            category: "company".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Download all 9 extra corpus sources and write results into the temp directories.
pub fn download_extra_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    // 8. USDA Foods
    on_progress("Downloading USDA Foods...");
    match download_and_process_foods(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/foods.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/foods.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("USDA Foods".into(), msg));
        }
    }

    // 9. IPEDS Institutions
    on_progress("Downloading IPEDS Institutions...");
    match download_and_process_institutions(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/institutions.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/institutions.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("IPEDS Institutions".into(), msg));
        }
    }

    // 10. IEEE OUI MAC Prefixes
    on_progress("Downloading IEEE OUI MAC Prefixes...");
    match download_and_process_mac_vendors(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/mac_vendors.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/mac_vendors.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("IEEE OUI MAC Prefixes".into(), msg));
        }
    }

    // 11. ISO 4217 Currencies
    on_progress("Downloading ISO 4217 Currencies...");
    match download_and_process_iso4217(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/currencies.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/currencies.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("ISO 4217 Currencies".into(), msg));
        }
    }

    // 12. SEC Stock Tickers
    on_progress("Downloading SEC Stock Tickers...");
    match download_and_process_stock_tickers(client, temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/stock_tickers.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/stock_tickers.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("SEC Stock Tickers".into(), msg));
        }
    }

    // 13. ICD-10 Codes
    on_progress("Downloading ICD-10 Codes...");
    match download_and_process_icd10(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/icd10_codes.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/icd10_codes.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("ICD-10 Codes".into(), msg));
        }
    }

    // 14. CLDR Locale Formats
    on_progress("Downloading CLDR Locale Formats...");
    match download_and_process_locale_formats(client, temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/locale_formats.json".into());
            report.total_size_bytes += size;
            on_progress(&format!(
                "  shared/locale_formats.json ({} KB)",
                size / 1024
            ));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("CLDR Locale Formats".into(), msg));
        }
    }

    // 15. Best Buy Products
    on_progress("Downloading Best Buy Products...");
    match download_and_process_products(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/products.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/products.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("Best Buy Products".into(), msg));
        }
    }

    // 16. Wikidata Companies
    on_progress("Downloading Wikidata Companies...");
    match download_and_process_companies(client, temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/companies.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/companies.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("Wikidata Companies".into(), msg));
        }
    }

    // 17. Open Food Facts
    on_progress("Downloading Open Food Facts (streaming ~1.2 GB, extracting 50K products)...");
    match download_and_process_open_food_facts(client, temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/food_products.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/food_products.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("Open Food Facts".into(), msg));
        }
    }

    // 18. German Handelsregister
    on_progress("Downloading German Handelsregister (streaming, extracting 10K companies)...");
    match download_and_process_german_companies(client, temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/german_companies.json".into());
            report.total_size_bytes += size;
            on_progress(&format!(
                "  shared/german_companies.json ({} KB)",
                size / 1024
            ));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("German Handelsregister".into(), msg));
        }
    }
}

// ---------------------------------------------------------------------------
// 8. USDA Foods
// ---------------------------------------------------------------------------

/// Download USDA FoodData Central foundation food CSV zip and produce foods.json.
fn download_and_process_foods(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_foundation_food_csv_2024-10-31.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip USDA foods: {e}")))?;

    // First pass: try to build a category lookup from food_category.csv
    let mut category_map: HashMap<String, String> = HashMap::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_lowercase();
        if fname.contains("food_category") && fname.ends_with(".csv") {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(file);
            for result in rdr.records() {
                if let Ok(record) = result {
                    // Expect columns: id, code, description (or similar)
                    if record.len() >= 2 {
                        let id = record[0].trim().to_string();
                        let desc = record
                            .get(record.len() - 1)
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        category_map.insert(id, desc);
                    }
                }
            }
            break;
        }
    }

    // Second pass: read food.csv
    let mut entries: Vec<FoodEntry> = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname_lower = file.name().to_lowercase();
        // Match files ending with exactly "/food.csv"
        if fname_lower.ends_with("/food.csv") || fname_lower == "food.csv" {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(file);

            // Find header indices
            let headers = rdr
                .headers()
                .map_err(|e| DatjitError::Corpus(format!("read USDA headers: {e}")))?
                .clone();
            let desc_idx = headers.iter().position(|h| h == "description");
            let data_type_idx = headers.iter().position(|h| h == "data_type");
            let cat_idx = headers.iter().position(|h| h == "food_category_id");

            for result in rdr.records() {
                if let Ok(record) = result {
                    // Filter: only Foundation rows if data_type column exists
                    if let Some(dt_idx) = data_type_idx {
                        if let Some(dt) = record.get(dt_idx) {
                            if dt != "foundation_food" {
                                continue;
                            }
                        }
                    }

                    let name = desc_idx
                        .and_then(|idx| record.get(idx))
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if name.is_empty() {
                        continue;
                    }

                    let category = cat_idx
                        .and_then(|idx| record.get(idx))
                        .and_then(|cid| category_map.get(cid.trim()))
                        .cloned()
                        .unwrap_or_default();

                    entries.push(FoodEntry { name, category });
                }
            }
            break;
        }
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize foods: {e}")))?;
    let path = dest_dir.join("foods.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write foods: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 9. IPEDS Institutions
// ---------------------------------------------------------------------------

fn map_control(val: &str) -> String {
    match val.trim() {
        "1" => "Public".into(),
        "2" => "Private nonprofit".into(),
        "3" => "Private for-profit".into(),
        _ => val.trim().to_string(),
    }
}

fn map_iclevel(val: &str) -> String {
    match val.trim() {
        "1" => "Four-year".into(),
        "2" => "Two-year".into(),
        "3" => "Less than two-year".into(),
        _ => val.trim().to_string(),
    }
}

/// Download IPEDS institution directory and produce institutions.json.
fn download_and_process_institutions(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://nces.ed.gov/ipeds/datacenter/data/HD2023.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip IPEDS: {e}")))?;

    let mut entries: Vec<InstitutionEntry> = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_lowercase();
        if !fname.ends_with(".csv") {
            continue;
        }

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        let headers = rdr
            .headers()
            .map_err(|e| DatjitError::Corpus(format!("read IPEDS headers: {e}")))?
            .clone();

        let name_idx = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("INSTNM"));
        let city_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("CITY"));
        let state_idx = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("STABBR"));
        let control_idx = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("CONTROL"));
        let level_idx = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("ICLEVEL"));

        for result in rdr.records() {
            if let Ok(record) = result {
                let name = name_idx
                    .and_then(|idx| record.get(idx))
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if name.is_empty() {
                    continue;
                }
                let city = city_idx
                    .and_then(|idx| record.get(idx))
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let state = state_idx
                    .and_then(|idx| record.get(idx))
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let control = control_idx
                    .and_then(|idx| record.get(idx))
                    .map(|v| map_control(v))
                    .unwrap_or_default();
                let level = level_idx
                    .and_then(|idx| record.get(idx))
                    .map(|v| map_iclevel(v))
                    .unwrap_or_default();

                entries.push(InstitutionEntry {
                    name,
                    city,
                    state,
                    control,
                    level,
                });
            }
        }
        break;
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize institutions: {e}")))?;
    let path = dest_dir.join("institutions.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write institutions: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 10. IEEE OUI MAC Prefixes
// ---------------------------------------------------------------------------

/// Download MAC vendor prefix CSV and produce mac_vendors.json.
fn download_and_process_mac_vendors(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://maclookup.app/downloads/csv-database/get-db",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<MacVendorEntry> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    for result in rdr.records() {
        if let Ok(record) = result {
            // Columns: mac_prefix, vendor_name, private, block_type, last_update
            if record.len() < 3 {
                continue;
            }
            let private_flag = record[2].trim();
            if private_flag == "1" || private_flag.eq_ignore_ascii_case("true") {
                continue;
            }
            let prefix = record[0].trim().to_string();
            let vendor = record[1].trim().to_string();
            if vendor.is_empty() {
                continue;
            }
            entries.push(MacVendorEntry { prefix, vendor });
            if entries.len() >= 5000 {
                break;
            }
        }
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize MAC vendors: {e}")))?;
    let path = dest_dir.join("mac_vendors.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write MAC vendors: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 11. ISO 4217 Currencies (XML)
// ---------------------------------------------------------------------------

/// Download ISO 4217 currency XML and produce currencies.json.
fn download_and_process_iso4217(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<Iso4217Entry> = Vec::new();
    let mut reader = Reader::from_str(&text);

    let mut in_ccy_ntry = false;
    let mut current_code = String::new();
    let mut current_name = String::new();
    let mut current_numeric = String::new();
    let mut current_minor: Option<u8> = None;
    let mut current_tag = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"CcyNtry" => {
                        in_ccy_ntry = true;
                        current_code.clear();
                        current_name.clear();
                        current_numeric.clear();
                        current_minor = None;
                    }
                    _ if in_ccy_ntry => {
                        current_tag = String::from_utf8_lossy(local.as_ref()).to_string();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_ccy_ntry {
                    let val = e.unescape().unwrap_or_default().trim().to_string();
                    match current_tag.as_str() {
                        "Ccy" => current_code = val,
                        "CcyNm" => current_name = val,
                        "CcyNbr" => current_numeric = val,
                        "CcyMnrUnts" => {
                            current_minor = val.parse::<u8>().ok();
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"CcyNtry" {
                    if !current_code.is_empty() {
                        entries.push(Iso4217Entry {
                            code: current_code.clone(),
                            name: current_name.clone(),
                            numeric_code: current_numeric.clone(),
                            minor_units: current_minor,
                        });
                    }
                    in_ccy_ntry = false;
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(DatjitError::Corpus(format!("parse ISO 4217 XML: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    // Deduplicate by currency code (same code can appear for multiple countries)
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(e.code.clone()));

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ISO 4217: {e}")))?;
    let path = dest_dir.join("currencies.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ISO 4217: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 12. SEC Stock Tickers
// ---------------------------------------------------------------------------

/// Download SEC company tickers JSON and produce stock_tickers.json.
/// SEC requires a User-Agent header with contact info.
/// Uses the exchange endpoint: { "fields": ["cik","name","ticker","exchange"], "data": [[...], ...] }
fn download_and_process_stock_tickers(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = "https://www.sec.gov/files/company_tickers_exchange.json";
    let resp = client
        .get(url)
        .header("User-Agent", "datjit/0.1.0 datjit@example.com")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download {url}: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for {url}",
            resp.status()
        )));
    }

    let body = resp
        .bytes()
        .map_err(|e| DatjitError::Corpus(format!("read response {url}: {e}")))?;

    let val: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| DatjitError::Corpus(format!("parse SEC JSON: {e}")))?;

    let mut entries: Vec<StockTickerEntry> = Vec::new();

    // New format: { "fields": ["cik","name","ticker","exchange"], "data": [[320193,"Apple Inc","AAPL","Nasdaq"], ...] }
    if let Some(data) = val.get("data").and_then(|v| v.as_array()) {
        for row in data {
            if let Some(arr) = row.as_array() {
                let cik = arr.first().and_then(|v| v.as_u64()).unwrap_or(0);
                let name = arr
                    .get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let ticker = arr
                    .get(2)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let exchange = arr
                    .get(3)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !ticker.is_empty() {
                    entries.push(StockTickerEntry {
                        ticker,
                        name,
                        cik,
                        exchange,
                    });
                }
            }
        }
    }

    entries.sort_by(|a, b| a.ticker.cmp(&b.ticker));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize stock tickers: {e}")))?;
    let path = dest_dir.join("stock_tickers.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write stock tickers: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 13. ICD-10 Codes
// ---------------------------------------------------------------------------

/// Download ICD-10 code descriptions and produce icd10_codes.json.
/// Tries CMS primary source first, then falls back to a GitHub CSV.
fn download_and_process_icd10(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Try primary: CMS zip file
    let primary_result = download_icd10_primary(client);

    let entries = match primary_result {
        Ok(e) => e,
        Err(_primary_err) => {
            // Fallback: GitHub CSV
            download_icd10_fallback(client)?
        }
    };

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ICD-10: {e}")))?;
    let path = dest_dir.join("icd10_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ICD-10: {e}")))?;

    Ok(json.len() as u64)
}

/// Try primary CMS zip source for ICD-10 codes.
fn download_icd10_primary(
    client: &reqwest::blocking::Client,
) -> Result<Vec<Icd10Entry>, DatjitError> {
    let data = download(
        client,
        "https://www.cms.gov/files/zip/2025-code-descriptions-tabular-order.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip ICD-10: {e}")))?;

    let mut entries: Vec<Icd10Entry> = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_lowercase();

        // Look for the text/tsv file with code descriptions
        if !fname.ends_with(".txt") && !fname.ends_with(".tsv") {
            continue;
        }

        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| DatjitError::Corpus(format!("read ICD-10 line: {e}")))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Try tab-separated first, then fixed-width (7-char code + space + description)
            let (code, description) = if trimmed.contains('\t') {
                let parts: Vec<&str> = trimmed.splitn(2, '\t').collect();
                if parts.len() >= 2 {
                    (parts[0].trim().to_string(), parts[1].trim().to_string())
                } else {
                    continue;
                }
            } else if trimmed.len() > 8 {
                // Fixed-width: code is first ~7 chars, rest is description
                let code_part = trimmed[..7].trim().to_string();
                let desc_part = trimmed[7..].trim().to_string();
                if code_part.is_empty() || desc_part.is_empty() {
                    continue;
                }
                (code_part, desc_part)
            } else {
                continue;
            };

            // ICD-10 codes start with a letter
            if !code.starts_with(|c: char| c.is_ascii_alphabetic()) {
                continue;
            }

            let category = if code.len() >= 3 {
                code[..3].to_string()
            } else {
                code.clone()
            };

            entries.push(Icd10Entry {
                code,
                description,
                category,
            });
        }
        if !entries.is_empty() {
            break;
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no ICD-10 entries found in CMS zip".into(),
        ));
    }

    Ok(entries)
}

/// Fallback: download ICD-10 codes from a GitHub CSV.
fn download_icd10_fallback(
    client: &reqwest::blocking::Client,
) -> Result<Vec<Icd10Entry>, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/k4m1113/ICD-10-CSV/master/codes.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<Icd10Entry> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    // Columns: Category Code, Diagnosis Code, Full Code, Abbreviated Description,
    //          Full Description, Category Title
    for result in rdr.records() {
        if let Ok(record) = result {
            // Use "Full Code" (index 2) and "Full Description" (index 4)
            let code = record.get(2).unwrap_or("").trim().to_string();
            let description = record.get(4).unwrap_or("").trim().to_string();
            if code.is_empty() || description.is_empty() {
                continue;
            }
            let category = if code.len() >= 3 {
                code[..3].to_string()
            } else {
                code.clone()
            };
            entries.push(Icd10Entry {
                code,
                description,
                category,
            });
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no ICD-10 entries found in fallback CSV".into(),
        ));
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// 14. CLDR Locale Formats
// ---------------------------------------------------------------------------

/// Download CLDR weekData JSON and produce locale_formats.json.
fn download_and_process_locale_formats(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/weekData.json",
    )?;

    let val: serde_json::Value = serde_json::from_slice(&data)
        .map_err(|e| DatjitError::Corpus(format!("parse CLDR weekData JSON: {e}")))?;

    let mut entries: Vec<LocaleFormatEntry> = Vec::new();

    // Navigate: supplemental -> weekData -> firstDay
    if let Some(first_day) = val
        .pointer("/supplemental/weekData/firstDay")
        .and_then(|v| v.as_object())
    {
        for (territory, day_val) in first_day {
            if let Some(day) = day_val.as_str() {
                entries.push(LocaleFormatEntry {
                    territory: territory.clone(),
                    first_day_of_week: day.to_string(),
                });
            }
        }
    }

    entries.sort_by(|a, b| a.territory.cmp(&b.territory));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize locale formats: {e}")))?;
    let path = dest_dir.join("locale_formats.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write locale formats: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 15. Best Buy Products
// ---------------------------------------------------------------------------

fn download_and_process_products(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/BestBuyAPIs/open-data-set/master/products.json",
    )?;
    let products: Vec<serde_json::Value> = serde_json::from_slice(&data)
        .map_err(|e| DatjitError::Corpus(format!("parse Best Buy JSON: {e}")))?;

    let entries: Vec<ProductEntry> = products
        .iter()
        .filter_map(|p| {
            let name = p.get("name")?.as_str()?.to_string();
            let sku = p
                .get("sku")
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
                .or_else(|| p.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();
            let price = p.get("price").and_then(|v| v.as_f64());
            let manufacturer = p
                .get("manufacturer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = p
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .chars()
                .take(200)
                .collect();
            let category = p
                .get("category")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.last())
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                return None;
            }
            Some(ProductEntry {
                name,
                sku,
                price,
                manufacturer,
                category,
                description,
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize products: {e}")))?;
    let path = dest_dir.join("products.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write products: {e}")))?;
    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 16. Wikidata Companies
// ---------------------------------------------------------------------------

fn download_and_process_companies(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let sparql = r#"SELECT DISTINCT ?companyLabel ?countryLabel ?industryLabel WHERE {
  ?company wdt:P31/wdt:P279* wd:Q4830453.
  ?company wdt:P17 ?country.
  OPTIONAL { ?company wdt:P452 ?industry. }
  SERVICE wikibase:label { bd:serviceParam wikibase:language "en". }
} LIMIT 10000"#;

    let resp = client
        .get("https://query.wikidata.org/sparql")
        .query(&[("format", "json"), ("query", sparql)])
        .header(
            "User-Agent",
            "datjit/0.1.0 (https://github.com/periplon/datjit)",
        )
        .send()
        .map_err(|e| DatjitError::Corpus(format!("wikidata query: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "Wikidata HTTP {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .map_err(|e| DatjitError::Corpus(format!("parse wikidata JSON: {e}")))?;

    let bindings = body
        .pointer("/results/bindings")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DatjitError::Corpus("invalid wikidata response".into()))?;

    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<CompanyEntry> = Vec::new();

    for binding in bindings {
        let name = binding
            .pointer("/companyLabel/value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() || name.starts_with('Q') || !seen.insert(name.clone()) {
            continue;
        }
        let country = binding
            .pointer("/countryLabel/value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let industry = binding
            .pointer("/industryLabel/value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        entries.push(CompanyEntry {
            name,
            country,
            industry,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize companies: {e}")))?;
    let path = dest_dir.join("companies.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write companies: {e}")))?;
    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// 17. Open Food Facts (streamed)
// ---------------------------------------------------------------------------

fn download_and_process_open_food_facts(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://static.openfoodfacts.org/data/en.openfoodfacts.org.products.csv.gz")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download Open Food Facts: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for Open Food Facts",
            resp.status()
        )));
    }

    let decoder = flate2::read::GzDecoder::new(resp);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b'\t')
        .from_reader(decoder);

    // Find column indices
    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("OFF headers: {e}")))?
        .clone();
    let name_idx = headers.iter().position(|h| h == "product_name");
    let brand_idx = headers.iter().position(|h| h == "brands");
    let cat_idx = headers
        .iter()
        .position(|h| h == "categories_en")
        .or_else(|| headers.iter().position(|h| h == "categories"));
    let country_idx = headers
        .iter()
        .position(|h| h == "countries_en")
        .or_else(|| headers.iter().position(|h| h == "countries"));
    let code_idx = headers.iter().position(|h| h == "code");

    let name_idx = name_idx.ok_or_else(|| {
        DatjitError::Corpus("Open Food Facts: missing product_name column".into())
    })?;

    let mut entries: Vec<FoodProductEntry> = Vec::new();
    let max_entries = 50_000;

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
        // Skip entries that are entirely non-ASCII (likely non-Latin scripts only)
        if !name.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        let brand = brand_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        let category = cat_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .chars()
            .take(100)
            .collect();
        let country = country_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .chars()
            .take(50)
            .collect();
        let barcode = code_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();

        entries.push(FoodProductEntry {
            name,
            brand,
            category,
            country,
            barcode,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize OFF: {e}")))?;
    let path = dest_dir.join("food_products.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write OFF: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 18. German Handelsregister (streamed)
// ---------------------------------------------------------------------------

fn download_and_process_german_companies(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://daten.offeneregister.de/de_companies_ocdata.jsonl.bz2")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download German register: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for German register",
            resp.status()
        )));
    }

    let decoder = bzip2::read::BzDecoder::new(resp);
    let reader = BufReader::new(decoder);

    let mut entries: Vec<GermanCompanyEntry> = Vec::new();
    let max_entries = 10_000;

    for line in reader.lines() {
        if entries.len() >= max_entries {
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }

        let obj: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }

        let city = obj
            .pointer("/registered_address/city")
            .or_else(|| obj.pointer("/registered_address/locality"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let state = obj
            .pointer("/registered_address/region")
            .or_else(|| obj.pointer("/registered_address/state"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let company_type = obj
            .get("company_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let status = obj
            .get("current_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        entries.push(GermanCompanyEntry {
            name,
            city,
            state,
            company_type,
            status,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize German register: {e}")))?;
    let path = dest_dir.join("german_companies.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write German register: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extra_known_sources_count() {
        let sources = extra_known_sources();
        assert_eq!(sources.len(), 11);
    }

    #[test]
    fn test_extra_known_sources_categories() {
        let sources = extra_known_sources();
        let categories: Vec<&str> = sources.iter().map(|s| s.category.as_str()).collect();
        assert!(categories.contains(&"shared"));
        assert!(categories.contains(&"product"));
        assert!(categories.contains(&"company"));
    }

    #[test]
    fn test_extra_known_sources_names() {
        let sources = extra_known_sources();
        let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"USDA Foods"));
        assert!(names.contains(&"IPEDS Institutions"));
        assert!(names.contains(&"IEEE OUI MAC Prefixes"));
        assert!(names.contains(&"ISO 4217 Currencies"));
        assert!(names.contains(&"SEC Stock Tickers"));
        assert!(names.contains(&"ICD-10 Codes"));
        assert!(names.contains(&"CLDR Locale Formats"));
        assert!(names.contains(&"Best Buy Open Products"));
        assert!(names.contains(&"Wikidata Companies"));
    }

    #[test]
    fn test_map_control() {
        assert_eq!(map_control("1"), "Public");
        assert_eq!(map_control("2"), "Private nonprofit");
        assert_eq!(map_control("3"), "Private for-profit");
        assert_eq!(map_control("99"), "99");
    }

    #[test]
    fn test_map_iclevel() {
        assert_eq!(map_iclevel("1"), "Four-year");
        assert_eq!(map_iclevel("2"), "Two-year");
        assert_eq!(map_iclevel("3"), "Less than two-year");
        assert_eq!(map_iclevel("0"), "0");
    }
}
