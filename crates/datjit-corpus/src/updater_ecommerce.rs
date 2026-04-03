//! Batch 6: Ecommerce datasets.
//!
//! Downloads product and category data from public domain sources:
//! - Instacart Market Basket Analysis (CC0): aisles, departments, products
//! - UK Online Retail dataset (UCI ML, Public Domain): product descriptions

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use calamine::Reader;
use datjit_core::error::DatjitError;
use serde::{Deserialize, Serialize};

use crate::updater::{download, download_source, CorpusSource, CorpusUpdateReport};

const INSTACART_RAW_BASE: &str =
    "https://raw.githubusercontent.com/khanhnamle1994/instacart-market-basket-analysis/master/";

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstacartAisleEntry {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstacartDepartmentEntry {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstacartProductEntry {
    pub name: String,
    pub aisle: String,
    pub department: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UkRetailProductEntry {
    pub description: String,
    pub unit_price: f64,
    pub country: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

pub fn ecommerce_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "Instacart Aisles".into(),
            description: "134 grocery aisles from Instacart Market Basket Analysis".into(),
            url: format!("{INSTACART_RAW_BASE}aisles.csv"),
            license: "CC0".into(),
            category: "ecommerce".into(),
        },
        CorpusSource {
            name: "Instacart Departments".into(),
            description: "21 grocery departments from Instacart Market Basket Analysis".into(),
            url: format!("{INSTACART_RAW_BASE}departments.csv"),
            license: "CC0".into(),
            category: "ecommerce".into(),
        },
        CorpusSource {
            name: "Instacart Products".into(),
            description: "~5K sampled grocery products with aisle/department from Instacart"
                .into(),
            url: format!("{INSTACART_RAW_BASE}products.csv"),
            license: "CC0".into(),
            category: "ecommerce".into(),
        },
        CorpusSource {
            name: "UK Online Retail".into(),
            description: "~4K unique product descriptions from UCI ML Repository".into(),
            url: "https://archive.ics.uci.edu/static/public/352/online+retail.zip".into(),
            license: "Public Domain".into(),
            category: "ecommerce".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub fn download_ecommerce_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    // Download reference tables first (needed for product join)
    download_source(
        "Instacart Aisles",
        "shared/instacart_aisles.json",
        || download_and_process_instacart_aisles(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Instacart Departments",
        "shared/instacart_departments.json",
        || download_and_process_instacart_departments(client, temp_shared),
        report,
        on_progress,
    );

    // Products needs aisles/departments JSON already written to temp_shared
    download_source(
        "Instacart Products",
        "shared/instacart_products.json",
        || download_and_process_instacart_products(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "UK Online Retail",
        "shared/uk_retail_products.json",
        || download_and_process_uk_retail(client, temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// Instacart aisles (134 rows, ~2KB CSV)
// ---------------------------------------------------------------------------

fn download_and_process_instacart_aisles(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(client, &format!("{INSTACART_RAW_BASE}aisles.csv"))?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut entries: Vec<InstacartAisleEntry> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| DatjitError::Corpus(format!("CSV parse: {e}")))?;
        let id: u32 = record
            .get(0)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let name = record.get(1).unwrap_or("").trim().to_string();
        if !name.is_empty() {
            entries.push(InstacartAisleEntry { id, name });
        }
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("instacart_aisles.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write instacart_aisles.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Instacart departments (21 rows, ~0.5KB CSV)
// ---------------------------------------------------------------------------

fn download_and_process_instacart_departments(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(client, &format!("{INSTACART_RAW_BASE}departments.csv"))?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut entries: Vec<InstacartDepartmentEntry> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| DatjitError::Corpus(format!("CSV parse: {e}")))?;
        let id: u32 = record
            .get(0)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let name = record.get(1).unwrap_or("").trim().to_string();
        if !name.is_empty() {
            entries.push(InstacartDepartmentEntry { id, name });
        }
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("instacart_departments.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write instacart_departments.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Instacart products (~50K rows, sample to ~5K)
// ---------------------------------------------------------------------------

fn download_and_process_instacart_products(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Load aisles and departments lookup maps from already-written JSON
    let aisles_map = load_id_name_map(&dest_dir.join("instacart_aisles.json"))?;
    let depts_map = load_id_name_map(&dest_dir.join("instacart_departments.json"))?;

    let data = download(client, &format!("{INSTACART_RAW_BASE}products.csv"))?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut entries: Vec<InstacartProductEntry> = Vec::new();
    let mut row_idx: usize = 0;
    let max_entries = 5000;

    for result in rdr.records() {
        if entries.len() >= max_entries {
            break;
        }
        let record = result.map_err(|e| DatjitError::Corpus(format!("CSV parse: {e}")))?;

        // Sample every 10th row
        row_idx += 1;
        if row_idx % 10 != 0 {
            continue;
        }

        // products.csv columns: product_id, product_name, aisle_id, department_id
        let name = record.get(1).unwrap_or("").trim().to_string();
        if name.is_empty() || name.len() < 3 {
            continue;
        }

        let aisle_id: u32 = record
            .get(2)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let dept_id: u32 = record
            .get(3)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let aisle = aisles_map
            .get(&aisle_id)
            .cloned()
            .unwrap_or_default();
        let department = depts_map
            .get(&dept_id)
            .cloned()
            .unwrap_or_default();

        entries.push(InstacartProductEntry {
            name,
            aisle,
            department,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("instacart_products.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write instacart_products.json: {e}")))?;

    Ok(json.len() as u64)
}

/// Load a JSON array of `{id, name}` objects into an id→name HashMap.
fn load_id_name_map(path: &Path) -> Result<HashMap<u32, String>, DatjitError> {
    let data =
        fs::read_to_string(path).map_err(|e| DatjitError::Corpus(format!("read {path:?}: {e}")))?;
    let arr: Vec<serde_json::Value> =
        serde_json::from_str(&data).map_err(|e| DatjitError::Corpus(format!("parse JSON: {e}")))?;

    let mut map = HashMap::new();
    for item in &arr {
        if let (Some(id), Some(name)) = (
            item.get("id").and_then(|v| v.as_u64()),
            item.get("name").and_then(|v| v.as_str()),
        ) {
            map.insert(id as u32, name.to_string());
        }
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// UK Online Retail (UCI ML, zip containing xlsx)
// ---------------------------------------------------------------------------

fn download_and_process_uk_retail(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://archive.ics.uci.edu/static/public/352/online+retail.zip",
    )?;

    // Unzip to find the xlsx file
    let cursor = std::io::Cursor::new(&data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| DatjitError::Corpus(format!("unzip: {e}")))?;

    // Find the xlsx file in the archive
    let xlsx_name = (0..archive.len())
        .filter_map(|i| {
            let f = archive.by_index(i).ok()?;
            let name = f.name().to_string();
            if name.ends_with(".xlsx") {
                Some(name)
            } else {
                None
            }
        })
        .next()
        .ok_or_else(|| DatjitError::Corpus("no .xlsx found in zip".into()))?;

    // Extract xlsx bytes
    let mut xlsx_file = archive
        .by_name(&xlsx_name)
        .map_err(|e| DatjitError::Corpus(format!("read xlsx from zip: {e}")))?;
    let mut xlsx_bytes = Vec::new();
    std::io::Read::read_to_end(&mut xlsx_file, &mut xlsx_bytes)
        .map_err(|e| DatjitError::Corpus(format!("read xlsx bytes: {e}")))?;

    // Parse xlsx with calamine
    let cursor = std::io::Cursor::new(&xlsx_bytes);
    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook_from_rs(cursor)
        .map_err(|e| DatjitError::Corpus(format!("open xlsx: {e}")))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| DatjitError::Corpus("no sheets in xlsx".into()))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| DatjitError::Corpus(format!("read sheet: {e}")))?;

    // Find column indices from header row
    let mut desc_col = None;
    let mut price_col = None;
    let mut country_col = None;

    if let Some(row) = range.rows().next() {
        for (i, cell) in row.iter().enumerate() {
            let header = format!("{cell}").to_lowercase();
            if header == "description" {
                desc_col = Some(i);
            } else if header == "unitprice" {
                price_col = Some(i);
            } else if header == "country" {
                country_col = Some(i);
            }
        }
    }

    let desc_col =
        desc_col.ok_or_else(|| DatjitError::Corpus("no Description column found".into()))?;

    // Extract unique products
    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<UkRetailProductEntry> = Vec::new();

    for row in range.rows().skip(1) {
        let description = row
            .get(desc_col)
            .map(|c: &calamine::Data| format!("{c}"))
            .unwrap_or_default()
            .trim()
            .to_string();

        if description.is_empty()
            || description.len() < 5
            || !description.chars().any(|ch: char| ch.is_ascii_alphabetic())
        {
            continue;
        }

        // Deduplicate by description
        if !seen.insert(description.clone()) {
            continue;
        }

        let unit_price = price_col
            .and_then(|i| row.get(i))
            .and_then(|c: &calamine::Data| match c {
                calamine::Data::Float(f) => Some(*f),
                calamine::Data::Int(n) => Some(*n as f64),
                _ => format!("{c}").parse::<f64>().ok(),
            })
            .unwrap_or(0.0);

        // Skip negative prices (returns/adjustments)
        if unit_price < 0.0 {
            continue;
        }

        let country = country_col
            .and_then(|i| row.get(i))
            .map(|c: &calamine::Data| format!("{c}"))
            .unwrap_or_default()
            .trim()
            .to_string();

        entries.push(UkRetailProductEntry {
            description,
            unit_price,
            country,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("uk_retail_products.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write uk_retail_products.json: {e}")))?;

    Ok(json.len() as u64)
}
