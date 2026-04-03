//! Batch 7: Taxonomy and classification system corpus sources.
//!
//! This module provides corpus sources for standard classification systems:
//! NAICS, ISIC, SIC, UNSPSC, HS, MCC, CIP, ISCED, ELF, LEI, COFOG, CPV,
//! and Dewey Decimal codes.

use std::fs;
use std::path::Path;

use datjit_core::error::DatjitError;
use serde::{Deserialize, Serialize};

use crate::updater::{download, download_source, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaicsEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsicEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SicEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnspscEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsCodeEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MccEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipEntry {
    pub code: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IscedEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElfEntry {
    pub code: String,
    pub name: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeiEntry {
    pub lei: String,
    pub name: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CofogEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpvEntry {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeweyEntry {
    pub code: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

/// Return the taxonomy corpus sources (Batch 7).
pub fn taxonomy_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "NAICS Industry Codes".into(),
            description: "North American Industry Classification System codes".into(),
            url: "https://data.bls.gov/cew/doc/titles/industry/industry_titles.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "ISIC Rev.4 Codes".into(),
            description: "International Standard Industrial Classification of All Economic Activities".into(),
            url: "https://unstats.un.org/unsd/classifications/Econ/Download/In%20Text/ISIC_Rev_4_english_structure.Txt".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "SIC Codes".into(),
            description: "Standard Industrial Classification codes".into(),
            url: "https://raw.githubusercontent.com/datasets/sic-codes/master/data/sic-codes.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "UNSPSC Codes".into(),
            description: "United Nations Standard Products and Services Code (segment level)".into(),
            url: "https://www.unspsc.org/".into(),
            license: "Proprietary (embedded segments only)".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "HS Codes".into(),
            description: "Harmonized System commodity classification codes".into(),
            url: "https://raw.githubusercontent.com/datasets/harmonized-system/master/data/harmonized-system.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "MCC Codes".into(),
            description: "Merchant Category Codes for payment card transactions".into(),
            url: "https://raw.githubusercontent.com/greggles/mcc-codes/main/mcc_codes.csv".into(),
            license: "MIT".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "CIP Codes".into(),
            description: "Classification of Instructional Programs codes".into(),
            url: "https://raw.githubusercontent.com/jkenlooper/cip-csv/master/cip.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "ISCED Codes".into(),
            description: "International Standard Classification of Education levels".into(),
            url: "https://uis.unesco.org/".into(),
            license: "Public Domain (embedded data)".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "ELF Codes".into(),
            description: "Entity Legal Forms code list (ISO 20275)".into(),
            url: "https://www.gleif.org/en/about-lei/code-lists/iso-20275-entity-legal-forms-code-list".into(),
            license: "GLEIF".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "LEI Registry".into(),
            description: "Legal Entity Identifier registry from GLEIF API".into(),
            url: "https://api.gleif.org/api/v1/lei-records".into(),
            license: "CC0".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "COFOG Codes".into(),
            description: "Classification of the Functions of Government".into(),
            url: "https://raw.githubusercontent.com/datasets/cofog/master/data/cofog.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "CPV Codes".into(),
            description: "Common Procurement Vocabulary codes for EU public procurement".into(),
            url: "https://raw.githubusercontent.com/open-contracting-extensions/european-union/main/codelists/cpv.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "Dewey Decimal Codes".into(),
            description: "Dewey Decimal Classification top-level divisions".into(),
            url: "https://www.oclc.org/dewey.html".into(),
            license: "Public Domain (embedded divisions)".into(),
            category: "taxonomy".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Download all taxonomy corpus sources and write results into the temp directories.
pub fn download_taxonomy_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "NAICS Industry Codes",
        "shared/naics_codes.json",
        || download_and_process_naics(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "ISIC Rev.4 Codes",
        "shared/isic_codes.json",
        || download_and_process_isic(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "SIC Codes",
        "shared/sic_codes.json",
        || download_and_process_sic(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "UNSPSC Codes",
        "shared/unspsc_codes.json",
        || download_and_process_unspsc(temp_shared),
        report,
        on_progress,
    );

    download_source(
        "HS Codes",
        "shared/hs_codes.json",
        || download_and_process_hs(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "MCC Codes",
        "shared/mcc_codes.json",
        || download_and_process_mcc(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "CIP Codes",
        "shared/cip_codes.json",
        || download_and_process_cip(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "ISCED Codes",
        "shared/isced_codes.json",
        || download_and_process_isced(temp_shared),
        report,
        on_progress,
    );

    download_source(
        "ELF Codes",
        "shared/elf_codes.json",
        || download_and_process_elf(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "LEI Registry",
        "shared/lei_registry.json",
        || download_and_process_lei(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "COFOG Codes",
        "shared/cofog_codes.json",
        || download_and_process_cofog(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "CPV Codes",
        "shared/cpv_codes.json",
        || download_and_process_cpv(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Dewey Decimal Codes",
        "shared/dewey_codes.json",
        || download_and_process_dewey(temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// 1. NAICS Codes (BLS)
// ---------------------------------------------------------------------------

fn download_and_process_naics(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://data.bls.gov/cew/doc/titles/industry/industry_titles.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read NAICS headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("industry_code") || h.to_lowercase().contains("code")
        })
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("industry_title") || h.to_lowercase().contains("title")
        })
        .unwrap_or(1);

    let mut entries: Vec<NaicsEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        entries.push(NaicsEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize NAICS: {e}")))?;
    let path = dest_dir.join("naics_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write NAICS: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 2. ISIC Rev.4
// ---------------------------------------------------------------------------

fn download_and_process_isic(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://unstats.un.org/unsd/classifications/Econ/Download/In%20Text/ISIC_Rev_4_english_structure.Txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<IsicEntry> = Vec::new();

    // Tab-separated text, columns are code and description
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b'\t')
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read ISIC headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("code"))
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("description") || h.to_lowercase().contains("title")
        })
        .unwrap_or(1);

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        entries.push(IsicEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ISIC: {e}")))?;
    let path = dest_dir.join("isic_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ISIC: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 3. SIC Codes
// ---------------------------------------------------------------------------

fn download_and_process_sic(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/datasets/sic-codes/master/data/sic-codes.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read SIC headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("code"))
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("description") || h.to_lowercase().contains("title")
        })
        .unwrap_or(1);

    let mut entries: Vec<SicEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        entries.push(SicEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize SIC: {e}")))?;
    let path = dest_dir.join("sic_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write SIC: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 4. UNSPSC Codes (embedded fallback)
// ---------------------------------------------------------------------------

fn download_and_process_unspsc(dest_dir: &Path) -> Result<u64, DatjitError> {
    let entries: Vec<UnspscEntry> = vec![
        UnspscEntry { code: "10000000".into(), description: "Live Plant and Animal Material".into() },
        UnspscEntry { code: "11000000".into(), description: "Mineral and Textile and Inedible Plant and Animal Materials".into() },
        UnspscEntry { code: "12000000".into(), description: "Chemicals including Bio Chemicals and Gas Materials".into() },
        UnspscEntry { code: "13000000".into(), description: "Resin and Rosin and Rubber and Foam and Film and Elastomeric Materials".into() },
        UnspscEntry { code: "14000000".into(), description: "Paper Materials and Products".into() },
        UnspscEntry { code: "15000000".into(), description: "Fuels and Fuel Additives and Lubricants and Anti corrosive Materials".into() },
        UnspscEntry { code: "20000000".into(), description: "Mining and Well Drilling Machinery and Accessories".into() },
        UnspscEntry { code: "21000000".into(), description: "Farming and Fishing and Forestry and Wildlife Machinery and Accessories".into() },
        UnspscEntry { code: "22000000".into(), description: "Building and Construction Machinery and Accessories".into() },
        UnspscEntry { code: "23000000".into(), description: "Industrial Manufacturing and Processing Machinery and Accessories".into() },
        UnspscEntry { code: "24000000".into(), description: "Material Handling and Conditioning and Storage Machinery and their Accessories and Supplies".into() },
        UnspscEntry { code: "25000000".into(), description: "Commercial and Military and Private Vehicles and their Accessories and Components".into() },
        UnspscEntry { code: "26000000".into(), description: "Power Generation and Distribution Machinery and Accessories".into() },
        UnspscEntry { code: "27000000".into(), description: "Tools and General Machinery".into() },
        UnspscEntry { code: "30000000".into(), description: "Structures and Building and Construction and Manufacturing Components and Supplies".into() },
        UnspscEntry { code: "31000000".into(), description: "Manufacturing Components and Supplies".into() },
        UnspscEntry { code: "32000000".into(), description: "Electronic Components and Supplies".into() },
        UnspscEntry { code: "39000000".into(), description: "Lighting Fixtures and Accessories".into() },
        UnspscEntry { code: "40000000".into(), description: "Distribution and Conditioning Systems and Equipment and Components".into() },
        UnspscEntry { code: "41000000".into(), description: "Laboratory and Measuring and Observing and Testing Equipment".into() },
        UnspscEntry { code: "42000000".into(), description: "Medical Equipment and Accessories and Supplies".into() },
        UnspscEntry { code: "43000000".into(), description: "Information Technology Broadcasting and Telecommunications".into() },
        UnspscEntry { code: "44000000".into(), description: "Office Equipment and Accessories and Supplies".into() },
        UnspscEntry { code: "45000000".into(), description: "Printing and Photographic and Audio and Visual Equipment and Supplies".into() },
        UnspscEntry { code: "46000000".into(), description: "Defense and Law Enforcement and Security and Safety Equipment and Supplies".into() },
        UnspscEntry { code: "47000000".into(), description: "Cleaning Equipment and Supplies".into() },
        UnspscEntry { code: "48000000".into(), description: "Service Industry Machinery and Equipment and Supplies".into() },
        UnspscEntry { code: "49000000".into(), description: "Sports and Recreational Equipment and Supplies and Accessories".into() },
        UnspscEntry { code: "50000000".into(), description: "Food Beverage and Tobacco Products".into() },
        UnspscEntry { code: "51000000".into(), description: "Drugs and Pharmaceutical Products".into() },
        UnspscEntry { code: "52000000".into(), description: "Domestic Appliances and Supplies and Consumer Electronic Products".into() },
        UnspscEntry { code: "53000000".into(), description: "Apparel and Luggage and Personal Care Products".into() },
        UnspscEntry { code: "54000000".into(), description: "Timepieces and Jewelry and Gemstone Products".into() },
        UnspscEntry { code: "55000000".into(), description: "Published Products".into() },
        UnspscEntry { code: "56000000".into(), description: "Furniture and Furnishings".into() },
        UnspscEntry { code: "60000000".into(), description: "Musical Instruments and Games and Toys and Arts and Crafts and Educational Equipment and Materials and Accessories and Supplies".into() },
        UnspscEntry { code: "70000000".into(), description: "Farming and Fishing and Forestry and Wildlife Contracting Services".into() },
        UnspscEntry { code: "71000000".into(), description: "Mining and Oil and Gas Services".into() },
        UnspscEntry { code: "72000000".into(), description: "Building and Facility Construction and Maintenance Services".into() },
        UnspscEntry { code: "73000000".into(), description: "Industrial Production and Manufacturing Services".into() },
        UnspscEntry { code: "76000000".into(), description: "Industrial Cleaning Services".into() },
        UnspscEntry { code: "77000000".into(), description: "Environmental Services".into() },
        UnspscEntry { code: "78000000".into(), description: "Transportation and Storage and Mail Services".into() },
        UnspscEntry { code: "80000000".into(), description: "Management and Business Professionals and Administrative Services".into() },
        UnspscEntry { code: "81000000".into(), description: "Engineering and Research and Technology Based Services".into() },
        UnspscEntry { code: "82000000".into(), description: "Editorial and Design and Graphic and Fine Art Services".into() },
        UnspscEntry { code: "83000000".into(), description: "Public Utilities and Public Sector Related Services".into() },
        UnspscEntry { code: "84000000".into(), description: "Financial and Insurance Services".into() },
        UnspscEntry { code: "85000000".into(), description: "Healthcare Services".into() },
        UnspscEntry { code: "86000000".into(), description: "Education and Training Services".into() },
        UnspscEntry { code: "90000000".into(), description: "Travel and Food and Lodging and Entertainment Services".into() },
        UnspscEntry { code: "91000000".into(), description: "Personal and Domestic Services".into() },
        UnspscEntry { code: "92000000".into(), description: "National Defense and Public Order and Security and Safety Services".into() },
        UnspscEntry { code: "93000000".into(), description: "Politics and Civic Affairs Services".into() },
        UnspscEntry { code: "94000000".into(), description: "Organizations and Clubs".into() },
        UnspscEntry { code: "95000000".into(), description: "Land and Buildings and Structures and Thoroughfares".into() },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize UNSPSC: {e}")))?;
    let path = dest_dir.join("unspsc_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write UNSPSC: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 5. HS Codes
// ---------------------------------------------------------------------------

fn download_and_process_hs(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/datasets/harmonized-system/master/data/harmonized-system.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read HS headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "id" || h.to_lowercase().contains("code"))
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("description") || h.to_lowercase().contains("text"))
        .unwrap_or(1);

    let mut entries: Vec<HsCodeEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        // Only keep 2 and 4-digit codes
        if code.len() > 4 {
            continue;
        }
        entries.push(HsCodeEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize HS: {e}")))?;
    let path = dest_dir.join("hs_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write HS: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 6. MCC Codes
// ---------------------------------------------------------------------------

fn download_and_process_mcc(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/greggles/mcc-codes/main/mcc_codes.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read MCC headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "mcc" || h.to_lowercase().contains("code"))
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("edited_description"))
        .or_else(|| {
            headers
                .iter()
                .position(|h| h.to_lowercase().contains("combined_description"))
        })
        .or_else(|| {
            headers
                .iter()
                .position(|h| h.to_lowercase().contains("description"))
        })
        .unwrap_or(1);

    let mut entries: Vec<MccEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        entries.push(MccEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize MCC: {e}")))?;
    let path = dest_dir.join("mcc_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write MCC: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 7. CIP Codes (try download, fall back to embedded)
// ---------------------------------------------------------------------------

fn download_and_process_cip(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Try downloading from GitHub first
    match download(
        client,
        "https://raw.githubusercontent.com/jkenlooper/cip-csv/master/cip.csv",
    ) {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data);
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(text.as_bytes());

            let headers = rdr
                .headers()
                .map_err(|e| DatjitError::Corpus(format!("read CIP headers: {e}")))?
                .clone();
            let code_idx = headers
                .iter()
                .position(|h| h.to_lowercase().contains("code") || h.to_lowercase().contains("cip"))
                .unwrap_or(0);
            let title_idx = headers
                .iter()
                .position(|h| {
                    h.to_lowercase().contains("title") || h.to_lowercase().contains("name")
                })
                .unwrap_or(1);

            let mut entries: Vec<CipEntry> = Vec::new();
            for result in rdr.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let code = record.get(code_idx).unwrap_or("").trim().to_string();
                let title = record.get(title_idx).unwrap_or("").trim().to_string();
                if code.is_empty() || title.is_empty() {
                    continue;
                }
                entries.push(CipEntry { code, title });
            }

            if !entries.is_empty() {
                let json = serde_json::to_string_pretty(&entries)
                    .map_err(|e| DatjitError::Corpus(format!("serialize CIP: {e}")))?;
                let path = dest_dir.join("cip_codes.json");
                fs::write(&path, &json)
                    .map_err(|e| DatjitError::Corpus(format!("write CIP: {e}")))?;
                return Ok(json.len() as u64);
            }
            // Fall through to embedded if no entries parsed
        }
        Err(_) => {
            // Fall through to embedded fallback
        }
    }

    // Embedded fallback: ~100 common CIP codes
    let entries: Vec<CipEntry> = vec![
        CipEntry {
            code: "01.0000".into(),
            title: "Agriculture, General".into(),
        },
        CipEntry {
            code: "01.0101".into(),
            title: "Agricultural Business and Management, General".into(),
        },
        CipEntry {
            code: "01.0901".into(),
            title: "Animal Sciences, General".into(),
        },
        CipEntry {
            code: "01.1001".into(),
            title: "Food Science".into(),
        },
        CipEntry {
            code: "01.1101".into(),
            title: "Plant Sciences, General".into(),
        },
        CipEntry {
            code: "03.0101".into(),
            title: "Natural Resources/Conservation, General".into(),
        },
        CipEntry {
            code: "03.0501".into(),
            title: "Forestry, General".into(),
        },
        CipEntry {
            code: "04.0201".into(),
            title: "Architecture".into(),
        },
        CipEntry {
            code: "05.0101".into(),
            title: "African-American/Black Studies".into(),
        },
        CipEntry {
            code: "05.0207".into(),
            title: "Women's Studies".into(),
        },
        CipEntry {
            code: "09.0100".into(),
            title: "Communication, General".into(),
        },
        CipEntry {
            code: "09.0401".into(),
            title: "Journalism".into(),
        },
        CipEntry {
            code: "09.0702".into(),
            title: "Digital Communication and Media/Multimedia".into(),
        },
        CipEntry {
            code: "11.0101".into(),
            title: "Computer and Information Sciences, General".into(),
        },
        CipEntry {
            code: "11.0201".into(),
            title: "Computer Programming/Programmer, General".into(),
        },
        CipEntry {
            code: "11.0401".into(),
            title: "Information Science/Studies".into(),
        },
        CipEntry {
            code: "11.0701".into(),
            title: "Computer Science".into(),
        },
        CipEntry {
            code: "11.0802".into(),
            title: "Data Modeling/Warehousing and Database Administration".into(),
        },
        CipEntry {
            code: "11.1003".into(),
            title: "Computer and Information Systems Security/Auditing/Information Assurance"
                .into(),
        },
        CipEntry {
            code: "13.0101".into(),
            title: "Education, General".into(),
        },
        CipEntry {
            code: "13.0401".into(),
            title: "Educational Leadership and Administration, General".into(),
        },
        CipEntry {
            code: "13.1001".into(),
            title: "Special Education and Teaching, General".into(),
        },
        CipEntry {
            code: "13.1202".into(),
            title: "Elementary Education and Teaching".into(),
        },
        CipEntry {
            code: "13.1205".into(),
            title: "Secondary Education and Teaching".into(),
        },
        CipEntry {
            code: "14.0101".into(),
            title: "Engineering, General".into(),
        },
        CipEntry {
            code: "14.0201".into(),
            title: "Aerospace, Aeronautical and Astronautical Engineering".into(),
        },
        CipEntry {
            code: "14.0301".into(),
            title: "Agricultural Engineering".into(),
        },
        CipEntry {
            code: "14.0501".into(),
            title: "Bioengineering and Biomedical Engineering".into(),
        },
        CipEntry {
            code: "14.0701".into(),
            title: "Chemical Engineering".into(),
        },
        CipEntry {
            code: "14.0801".into(),
            title: "Civil Engineering, General".into(),
        },
        CipEntry {
            code: "14.0901".into(),
            title: "Computer Engineering, General".into(),
        },
        CipEntry {
            code: "14.1001".into(),
            title: "Electrical and Electronics Engineering".into(),
        },
        CipEntry {
            code: "14.1301".into(),
            title: "Engineering Science".into(),
        },
        CipEntry {
            code: "14.1401".into(),
            title: "Environmental/Environmental Health Engineering".into(),
        },
        CipEntry {
            code: "14.1801".into(),
            title: "Materials Engineering".into(),
        },
        CipEntry {
            code: "14.1901".into(),
            title: "Mechanical Engineering".into(),
        },
        CipEntry {
            code: "14.2701".into(),
            title: "Systems Engineering".into(),
        },
        CipEntry {
            code: "15.0000".into(),
            title: "Engineering Technology, General".into(),
        },
        CipEntry {
            code: "16.0101".into(),
            title: "Foreign Languages and Literatures, General".into(),
        },
        CipEntry {
            code: "16.0501".into(),
            title: "German Language and Literature".into(),
        },
        CipEntry {
            code: "16.0901".into(),
            title: "French Language and Literature".into(),
        },
        CipEntry {
            code: "16.0905".into(),
            title: "Spanish Language and Literature".into(),
        },
        CipEntry {
            code: "19.0101".into(),
            title: "Family and Consumer Sciences/Human Sciences, General".into(),
        },
        CipEntry {
            code: "22.0101".into(),
            title: "Law".into(),
        },
        CipEntry {
            code: "22.0302".into(),
            title: "Legal Assistant/Paralegal".into(),
        },
        CipEntry {
            code: "23.0101".into(),
            title: "English Language and Literature, General".into(),
        },
        CipEntry {
            code: "23.1302".into(),
            title: "Creative Writing".into(),
        },
        CipEntry {
            code: "24.0101".into(),
            title: "Liberal Arts and Sciences/Liberal Studies".into(),
        },
        CipEntry {
            code: "25.0101".into(),
            title: "Library Science/Librarianship".into(),
        },
        CipEntry {
            code: "26.0101".into(),
            title: "Biology/Biological Sciences, General".into(),
        },
        CipEntry {
            code: "26.0202".into(),
            title: "Biochemistry".into(),
        },
        CipEntry {
            code: "26.0406".into(),
            title: "Cell/Cellular Biology and Anatomical Sciences".into(),
        },
        CipEntry {
            code: "26.0502".into(),
            title: "Microbiology, General".into(),
        },
        CipEntry {
            code: "26.0802".into(),
            title: "Genetics, General".into(),
        },
        CipEntry {
            code: "26.0908".into(),
            title: "Exercise Physiology".into(),
        },
        CipEntry {
            code: "26.1301".into(),
            title: "Ecology".into(),
        },
        CipEntry {
            code: "27.0101".into(),
            title: "Mathematics, General".into(),
        },
        CipEntry {
            code: "27.0501".into(),
            title: "Statistics, General".into(),
        },
        CipEntry {
            code: "27.0301".into(),
            title: "Applied Mathematics, General".into(),
        },
        CipEntry {
            code: "30.0501".into(),
            title: "Peace Studies and Conflict Resolution".into(),
        },
        CipEntry {
            code: "30.1701".into(),
            title: "Behavioral Sciences".into(),
        },
        CipEntry {
            code: "30.1801".into(),
            title: "Natural Sciences".into(),
        },
        CipEntry {
            code: "31.0501".into(),
            title: "Health and Physical Education/Fitness, General".into(),
        },
        CipEntry {
            code: "38.0101".into(),
            title: "Philosophy".into(),
        },
        CipEntry {
            code: "38.0201".into(),
            title: "Religion/Religious Studies".into(),
        },
        CipEntry {
            code: "39.0201".into(),
            title: "Bible/Biblical Studies".into(),
        },
        CipEntry {
            code: "40.0101".into(),
            title: "Physical Sciences".into(),
        },
        CipEntry {
            code: "40.0501".into(),
            title: "Chemistry, General".into(),
        },
        CipEntry {
            code: "40.0601".into(),
            title: "Geology/Earth Science, General".into(),
        },
        CipEntry {
            code: "40.0801".into(),
            title: "Physics, General".into(),
        },
        CipEntry {
            code: "42.0101".into(),
            title: "Psychology, General".into(),
        },
        CipEntry {
            code: "42.2804".into(),
            title: "Industrial and Organizational Psychology".into(),
        },
        CipEntry {
            code: "42.2803".into(),
            title: "Counseling Psychology".into(),
        },
        CipEntry {
            code: "43.0103".into(),
            title: "Criminal Justice/Law Enforcement Administration".into(),
        },
        CipEntry {
            code: "43.0104".into(),
            title: "Criminal Justice/Safety Studies".into(),
        },
        CipEntry {
            code: "44.0401".into(),
            title: "Public Administration".into(),
        },
        CipEntry {
            code: "44.0701".into(),
            title: "Social Work".into(),
        },
        CipEntry {
            code: "45.0201".into(),
            title: "Anthropology".into(),
        },
        CipEntry {
            code: "45.0601".into(),
            title: "Economics, General".into(),
        },
        CipEntry {
            code: "45.0701".into(),
            title: "Geography".into(),
        },
        CipEntry {
            code: "45.0901".into(),
            title: "International Relations and Affairs".into(),
        },
        CipEntry {
            code: "45.1001".into(),
            title: "Political Science and Government, General".into(),
        },
        CipEntry {
            code: "45.1101".into(),
            title: "Sociology".into(),
        },
        CipEntry {
            code: "50.0301".into(),
            title: "Dance, General".into(),
        },
        CipEntry {
            code: "50.0401".into(),
            title: "Design and Visual Communications, General".into(),
        },
        CipEntry {
            code: "50.0501".into(),
            title: "Drama and Dramatics/Theatre Arts, General".into(),
        },
        CipEntry {
            code: "50.0601".into(),
            title: "Film/Cinema/Video Studies".into(),
        },
        CipEntry {
            code: "50.0702".into(),
            title: "Fine/Studio Arts, General".into(),
        },
        CipEntry {
            code: "50.0901".into(),
            title: "Music, General".into(),
        },
        CipEntry {
            code: "50.0903".into(),
            title: "Music Performance, General".into(),
        },
        CipEntry {
            code: "50.1001".into(),
            title: "Art/Art Studies, General".into(),
        },
        CipEntry {
            code: "51.0000".into(),
            title: "Health Services/Allied Health/Health Sciences, General".into(),
        },
        CipEntry {
            code: "51.0201".into(),
            title: "Communication Sciences and Disorders, General".into(),
        },
        CipEntry {
            code: "51.0601".into(),
            title: "Dental Hygiene/Hygienist".into(),
        },
        CipEntry {
            code: "51.0701".into(),
            title: "Health/Health Care Administration/Management".into(),
        },
        CipEntry {
            code: "51.0913".into(),
            title: "Athletic Training/Trainer".into(),
        },
        CipEntry {
            code: "51.1501".into(),
            title: "Mental Health Counseling/Counselor".into(),
        },
        CipEntry {
            code: "51.2201".into(),
            title: "Public Health, General".into(),
        },
        CipEntry {
            code: "51.3801".into(),
            title: "Registered Nursing/Registered Nurse".into(),
        },
        CipEntry {
            code: "52.0101".into(),
            title: "Business/Commerce, General".into(),
        },
        CipEntry {
            code: "52.0201".into(),
            title: "Business Administration and Management, General".into(),
        },
        CipEntry {
            code: "52.0301".into(),
            title: "Accounting".into(),
        },
        CipEntry {
            code: "52.0801".into(),
            title: "Finance, General".into(),
        },
        CipEntry {
            code: "52.1001".into(),
            title: "Human Resources Management/Personnel Administration, General".into(),
        },
        CipEntry {
            code: "52.1201".into(),
            title: "Management Information Systems, General".into(),
        },
        CipEntry {
            code: "52.1401".into(),
            title: "Marketing/Marketing Management, General".into(),
        },
        CipEntry {
            code: "52.1501".into(),
            title: "Real Estate".into(),
        },
        CipEntry {
            code: "54.0101".into(),
            title: "History, General".into(),
        },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize CIP: {e}")))?;
    let path = dest_dir.join("cip_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write CIP: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 8. ISCED Codes (embedded)
// ---------------------------------------------------------------------------

fn download_and_process_isced(dest_dir: &Path) -> Result<u64, DatjitError> {
    let entries: Vec<IscedEntry> = vec![
        IscedEntry {
            code: "0".into(),
            description: "Early childhood education".into(),
        },
        IscedEntry {
            code: "01".into(),
            description: "Early childhood educational development".into(),
        },
        IscedEntry {
            code: "02".into(),
            description: "Pre-primary education".into(),
        },
        IscedEntry {
            code: "1".into(),
            description: "Primary education".into(),
        },
        IscedEntry {
            code: "10".into(),
            description: "Primary education".into(),
        },
        IscedEntry {
            code: "2".into(),
            description: "Lower secondary education".into(),
        },
        IscedEntry {
            code: "21".into(),
            description: "General lower secondary".into(),
        },
        IscedEntry {
            code: "22".into(),
            description: "Vocational lower secondary".into(),
        },
        IscedEntry {
            code: "23".into(),
            description: "Combined lower secondary".into(),
        },
        IscedEntry {
            code: "24".into(),
            description: "Orientation unspecified lower secondary".into(),
        },
        IscedEntry {
            code: "25".into(),
            description: "Lower secondary education sufficient for partial level completion".into(),
        },
        IscedEntry {
            code: "3".into(),
            description: "Upper secondary education".into(),
        },
        IscedEntry {
            code: "31".into(),
            description: "General upper secondary".into(),
        },
        IscedEntry {
            code: "32".into(),
            description: "Vocational upper secondary".into(),
        },
        IscedEntry {
            code: "33".into(),
            description: "Combined upper secondary".into(),
        },
        IscedEntry {
            code: "34".into(),
            description: "Orientation unspecified upper secondary".into(),
        },
        IscedEntry {
            code: "35".into(),
            description: "Upper secondary partial level completion".into(),
        },
        IscedEntry {
            code: "4".into(),
            description: "Post-secondary non-tertiary education".into(),
        },
        IscedEntry {
            code: "41".into(),
            description: "General post-secondary non-tertiary".into(),
        },
        IscedEntry {
            code: "42".into(),
            description: "Vocational post-secondary non-tertiary".into(),
        },
        IscedEntry {
            code: "43".into(),
            description: "Combined post-secondary non-tertiary".into(),
        },
        IscedEntry {
            code: "44".into(),
            description: "Orientation unspecified post-secondary non-tertiary".into(),
        },
        IscedEntry {
            code: "5".into(),
            description: "Short-cycle tertiary education".into(),
        },
        IscedEntry {
            code: "51".into(),
            description: "General short-cycle tertiary".into(),
        },
        IscedEntry {
            code: "52".into(),
            description: "Vocational short-cycle tertiary".into(),
        },
        IscedEntry {
            code: "53".into(),
            description: "Combined short-cycle tertiary".into(),
        },
        IscedEntry {
            code: "54".into(),
            description: "Orientation unspecified short-cycle tertiary".into(),
        },
        IscedEntry {
            code: "6".into(),
            description: "Bachelor's or equivalent level".into(),
        },
        IscedEntry {
            code: "61".into(),
            description: "Academic bachelor".into(),
        },
        IscedEntry {
            code: "62".into(),
            description: "Professional bachelor".into(),
        },
        IscedEntry {
            code: "63".into(),
            description: "Combined bachelor".into(),
        },
        IscedEntry {
            code: "64".into(),
            description: "Orientation unspecified bachelor".into(),
        },
        IscedEntry {
            code: "65".into(),
            description: "First degree or bachelor insufficient for ISCED 7".into(),
        },
        IscedEntry {
            code: "7".into(),
            description: "Master's or equivalent level".into(),
        },
        IscedEntry {
            code: "71".into(),
            description: "Academic master".into(),
        },
        IscedEntry {
            code: "72".into(),
            description: "Professional master".into(),
        },
        IscedEntry {
            code: "73".into(),
            description: "Combined master".into(),
        },
        IscedEntry {
            code: "74".into(),
            description: "Orientation unspecified master".into(),
        },
        IscedEntry {
            code: "75".into(),
            description: "Long first degree or master insufficient for ISCED 8".into(),
        },
        IscedEntry {
            code: "76".into(),
            description: "Long first degree or master sufficient for ISCED 8".into(),
        },
        IscedEntry {
            code: "8".into(),
            description: "Doctoral or equivalent level".into(),
        },
        IscedEntry {
            code: "81".into(),
            description: "Academic doctoral".into(),
        },
        IscedEntry {
            code: "82".into(),
            description: "Professional doctoral".into(),
        },
        IscedEntry {
            code: "83".into(),
            description: "Combined doctoral".into(),
        },
        IscedEntry {
            code: "84".into(),
            description: "Orientation unspecified doctoral".into(),
        },
        IscedEntry {
            code: "9".into(),
            description: "Not elsewhere classified".into(),
        },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ISCED: {e}")))?;
    let path = dest_dir.join("isced_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ISCED: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 9. ELF Codes (try download, fall back to embedded)
// ---------------------------------------------------------------------------

fn download_and_process_elf(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Try downloading from GLEIF
    match download(
        client,
        "https://www.gleif.org/content/2-about-lei/6-code-lists/2-iso-20275-entity-legal-forms-code-list/2024-11-28_elf-code-list-v1.5.csv",
    ) {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data);
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(text.as_bytes());

            let headers = rdr
                .headers()
                .map_err(|e| DatjitError::Corpus(format!("read ELF headers: {e}")))?
                .clone();
            let code_idx = headers
                .iter()
                .position(|h| h.to_lowercase().contains("elf code") || h.to_lowercase().contains("code"))
                .unwrap_or(0);
            let name_idx = headers
                .iter()
                .position(|h| {
                    h.to_lowercase().contains("entity legal form name")
                        || h.to_lowercase().contains("local name")
                        || h.to_lowercase().contains("name")
                })
                .unwrap_or(1);
            let country_idx = headers
                .iter()
                .position(|h| h.to_lowercase().contains("country"))
                .unwrap_or(2);

            let mut entries: Vec<ElfEntry> = Vec::new();
            for result in rdr.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let code = record.get(code_idx).unwrap_or("").trim().to_string();
                let name = record.get(name_idx).unwrap_or("").trim().to_string();
                let country = record.get(country_idx).unwrap_or("").trim().to_string();
                if code.is_empty() || name.is_empty() {
                    continue;
                }
                entries.push(ElfEntry { code, name, country });
            }

            if !entries.is_empty() {
                let json = serde_json::to_string_pretty(&entries)
                    .map_err(|e| DatjitError::Corpus(format!("serialize ELF: {e}")))?;
                let path = dest_dir.join("elf_codes.json");
                fs::write(&path, &json)
                    .map_err(|e| DatjitError::Corpus(format!("write ELF: {e}")))?;
                return Ok(json.len() as u64);
            }
            // Fall through to embedded if no entries parsed
        }
        Err(_) => {
            // Fall through to embedded fallback
        }
    }

    // Embedded fallback: ~50 common ELF codes
    let entries: Vec<ElfEntry> = vec![
        ElfEntry {
            code: "2HBR".into(),
            name: "Public Limited Company".into(),
            country: "GB".into(),
        },
        ElfEntry {
            code: "8888".into(),
            name: "Limited Liability Company".into(),
            country: "US".into(),
        },
        ElfEntry {
            code: "XTIQ".into(),
            name: "Corporation".into(),
            country: "US".into(),
        },
        ElfEntry {
            code: "QK3M".into(),
            name: "Gesellschaft mit beschrankter Haftung".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "V2YH".into(),
            name: "Aktiengesellschaft".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "PNBS".into(),
            name: "Societe a responsabilite limitee".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "HPWX".into(),
            name: "Societe anonyme".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "D4SE".into(),
            name: "Societe par actions simplifiee".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "9GBM".into(),
            name: "Sociedad Limitada".into(),
            country: "ES".into(),
        },
        ElfEntry {
            code: "CUXT".into(),
            name: "Sociedad Anonima".into(),
            country: "ES".into(),
        },
        ElfEntry {
            code: "CXHF".into(),
            name: "Societa a responsabilita limitata".into(),
            country: "IT".into(),
        },
        ElfEntry {
            code: "MGRM".into(),
            name: "Societa per azioni".into(),
            country: "IT".into(),
        },
        ElfEntry {
            code: "W2S7".into(),
            name: "Besloten vennootschap".into(),
            country: "NL".into(),
        },
        ElfEntry {
            code: "B4RV".into(),
            name: "Naamloze vennootschap".into(),
            country: "NL".into(),
        },
        ElfEntry {
            code: "VFNP".into(),
            name: "Aktiebolag".into(),
            country: "SE".into(),
        },
        ElfEntry {
            code: "QKYE".into(),
            name: "Aksjeselskap".into(),
            country: "NO".into(),
        },
        ElfEntry {
            code: "3NE2".into(),
            name: "Anpartsselskab".into(),
            country: "DK".into(),
        },
        ElfEntry {
            code: "PUHL".into(),
            name: "Aktieselskab".into(),
            country: "DK".into(),
        },
        ElfEntry {
            code: "8Z6H".into(),
            name: "Osakeyhti".into(),
            country: "FI".into(),
        },
        ElfEntry {
            code: "JG75".into(),
            name: "Kabushiki Kaisha".into(),
            country: "JP".into(),
        },
        ElfEntry {
            code: "TDTD".into(),
            name: "Godo Kaisha".into(),
            country: "JP".into(),
        },
        ElfEntry {
            code: "N1FB".into(),
            name: "Limited Partnership".into(),
            country: "US".into(),
        },
        ElfEntry {
            code: "EWZZ".into(),
            name: "General Partnership".into(),
            country: "US".into(),
        },
        ElfEntry {
            code: "VDT3".into(),
            name: "Sole Proprietorship".into(),
            country: "US".into(),
        },
        ElfEntry {
            code: "4NKL".into(),
            name: "Limited Liability Partnership".into(),
            country: "GB".into(),
        },
        ElfEntry {
            code: "F7TI".into(),
            name: "Private Limited Company".into(),
            country: "GB".into(),
        },
        ElfEntry {
            code: "3UYX".into(),
            name: "Community Interest Company".into(),
            country: "GB".into(),
        },
        ElfEntry {
            code: "ZZPJ".into(),
            name: "Gesellschaft burgerlichen Rechts".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "1WRN".into(),
            name: "Kommanditgesellschaft".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "OD68".into(),
            name: "Offene Handelsgesellschaft".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "9HJR".into(),
            name: "Eingetragener Verein".into(),
            country: "DE".into(),
        },
        ElfEntry {
            code: "PI7A".into(),
            name: "Stichting".into(),
            country: "NL".into(),
        },
        ElfEntry {
            code: "BP7X".into(),
            name: "Cooperatieve Vereniging".into(),
            country: "NL".into(),
        },
        ElfEntry {
            code: "K7RB".into(),
            name: "Societe en nom collectif".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "M21Y".into(),
            name: "Societe civile".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "66QJ".into(),
            name: "Societe en commandite simple".into(),
            country: "FR".into(),
        },
        ElfEntry {
            code: "YRFV".into(),
            name: "Sociedade Limitada".into(),
            country: "BR".into(),
        },
        ElfEntry {
            code: "AZHJ".into(),
            name: "Sociedade Anonima".into(),
            country: "BR".into(),
        },
        ElfEntry {
            code: "T27E".into(),
            name: "Proprietary Limited Company".into(),
            country: "AU".into(),
        },
        ElfEntry {
            code: "72BG".into(),
            name: "Public Company Limited by Shares".into(),
            country: "AU".into(),
        },
        ElfEntry {
            code: "DSBK".into(),
            name: "Inc.".into(),
            country: "CA".into(),
        },
        ElfEntry {
            code: "W7DF".into(),
            name: "Limited".into(),
            country: "CA".into(),
        },
        ElfEntry {
            code: "R9GZ".into(),
            name: "Gesellschaft mit beschrankter Haftung".into(),
            country: "AT".into(),
        },
        ElfEntry {
            code: "1ZQ6".into(),
            name: "Aktiengesellschaft".into(),
            country: "AT".into(),
        },
        ElfEntry {
            code: "H8PF".into(),
            name: "Gesellschaft mit beschrankter Haftung".into(),
            country: "CH".into(),
        },
        ElfEntry {
            code: "5MGR".into(),
            name: "Aktiengesellschaft".into(),
            country: "CH".into(),
        },
        ElfEntry {
            code: "D3KW".into(),
            name: "Private Company Limited by Shares".into(),
            country: "IE".into(),
        },
        ElfEntry {
            code: "E46D".into(),
            name: "Spolka z ograniczona odpowiedzialnoscia".into(),
            country: "PL".into(),
        },
        ElfEntry {
            code: "J3JI".into(),
            name: "Spolka akcyjna".into(),
            country: "PL".into(),
        },
        ElfEntry {
            code: "6AGR".into(),
            name: "Sociedade por Quotas".into(),
            country: "PT".into(),
        },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ELF: {e}")))?;
    let path = dest_dir.join("elf_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ELF: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 10. LEI Registry (paginated API)
// ---------------------------------------------------------------------------

fn download_and_process_lei(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let mut entries: Vec<LeiEntry> = Vec::new();

    for page in 1..=50 {
        let url = format!(
            "https://api.gleif.org/api/v1/lei-records?page[size]=100&page[number]={}",
            page
        );
        let data = match download(client, &url) {
            Ok(d) => d,
            Err(_) => break,
        };

        let parsed: serde_json::Value = serde_json::from_slice(&data)
            .map_err(|e| DatjitError::Corpus(format!("parse LEI page {page}: {e}")))?;

        let records = match parsed.get("data").and_then(|d| d.as_array()) {
            Some(arr) if !arr.is_empty() => arr,
            _ => break,
        };

        for record in records {
            let attributes = match record.get("attributes") {
                Some(a) => a,
                None => continue,
            };

            let lei = attributes
                .get("lei")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            let entity = match attributes.get("entity") {
                Some(e) => e,
                None => continue,
            };

            let name = entity
                .get("legalName")
                .and_then(|ln| ln.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            let country = entity
                .get("jurisdiction")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            if lei.is_empty() || name.is_empty() {
                continue;
            }

            entries.push(LeiEntry { lei, name, country });
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no LEI entries retrieved from GLEIF API".into(),
        ));
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize LEI: {e}")))?;
    let path = dest_dir.join("lei_registry.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write LEI: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 11. COFOG Codes
// ---------------------------------------------------------------------------

fn download_and_process_cofog(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/datasets/cofog/master/data/cofog.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read COFOG headers: {e}")))?
        .clone();
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase().contains("code"))
        .unwrap_or(0);
    let desc_idx = headers
        .iter()
        .position(|h| {
            h.to_lowercase().contains("description") || h.to_lowercase().contains("title")
        })
        .unwrap_or(1);

    let mut entries: Vec<CofogEntry> = Vec::new();
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let description = record.get(desc_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || description.is_empty() {
            continue;
        }
        entries.push(CofogEntry { code, description });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize COFOG: {e}")))?;
    let path = dest_dir.join("cofog_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write COFOG: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 12. CPV Codes (try download, fall back to embedded)
// ---------------------------------------------------------------------------

fn download_and_process_cpv(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Try downloading from GitHub
    match download(
        client,
        "https://raw.githubusercontent.com/open-contracting-extensions/european-union/main/codelists/cpv.csv",
    ) {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data);
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(text.as_bytes());

            let headers = rdr
                .headers()
                .map_err(|e| DatjitError::Corpus(format!("read CPV headers: {e}")))?
                .clone();
            let code_idx = headers
                .iter()
                .position(|h| h.to_lowercase().contains("code"))
                .unwrap_or(0);
            let desc_idx = headers
                .iter()
                .position(|h| {
                    h.to_lowercase().contains("description")
                        || h.to_lowercase().contains("title")
                        || h.to_lowercase().contains("name")
                })
                .unwrap_or(1);

            let mut entries: Vec<CpvEntry> = Vec::new();
            for result in rdr.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let code = record.get(code_idx).unwrap_or("").trim().to_string();
                let description = record.get(desc_idx).unwrap_or("").trim().to_string();
                if code.is_empty() || description.is_empty() {
                    continue;
                }
                entries.push(CpvEntry { code, description });
            }

            if !entries.is_empty() {
                let json = serde_json::to_string_pretty(&entries)
                    .map_err(|e| DatjitError::Corpus(format!("serialize CPV: {e}")))?;
                let path = dest_dir.join("cpv_codes.json");
                fs::write(&path, &json)
                    .map_err(|e| DatjitError::Corpus(format!("write CPV: {e}")))?;
                return Ok(json.len() as u64);
            }
            // Fall through to embedded if no entries parsed
        }
        Err(_) => {
            // Fall through to embedded fallback
        }
    }

    // Embedded fallback: top-level CPV divisions
    let entries: Vec<CpvEntry> = vec![
        CpvEntry { code: "03000000".into(), description: "Agricultural, farming, fishing, forestry and related products".into() },
        CpvEntry { code: "09000000".into(), description: "Petroleum products, fuel, electricity and other sources of energy".into() },
        CpvEntry { code: "14000000".into(), description: "Mining, basic metals and related products".into() },
        CpvEntry { code: "15000000".into(), description: "Food, beverages, tobacco and related products".into() },
        CpvEntry { code: "16000000".into(), description: "Agricultural machinery".into() },
        CpvEntry { code: "18000000".into(), description: "Clothing, footwear, luggage articles and accessories".into() },
        CpvEntry { code: "19000000".into(), description: "Leather and textile fabrics, plastic and rubber materials".into() },
        CpvEntry { code: "22000000".into(), description: "Printed matter and related products".into() },
        CpvEntry { code: "24000000".into(), description: "Chemical products".into() },
        CpvEntry { code: "30000000".into(), description: "Office and computing machinery, equipment and supplies except furniture and software packages".into() },
        CpvEntry { code: "31000000".into(), description: "Electrical machinery, apparatus, equipment and consumables; lighting".into() },
        CpvEntry { code: "32000000".into(), description: "Radio, television, communication, telecommunication and related equipment".into() },
        CpvEntry { code: "33000000".into(), description: "Medical equipments, pharmaceuticals and personal care products".into() },
        CpvEntry { code: "34000000".into(), description: "Transport equipment and auxiliary products to transportation".into() },
        CpvEntry { code: "35000000".into(), description: "Security, fire-fighting, police and defence equipment".into() },
        CpvEntry { code: "37000000".into(), description: "Musical instruments, sport goods, games, toys, handicraft, art materials and accessories".into() },
        CpvEntry { code: "38000000".into(), description: "Laboratory, optical and precision equipments (excl. glasses)".into() },
        CpvEntry { code: "39000000".into(), description: "Furniture (incl. office furniture), furnishings, domestic appliances (excl. lighting) and cleaning products".into() },
        CpvEntry { code: "41000000".into(), description: "Collected and purified water".into() },
        CpvEntry { code: "42000000".into(), description: "Industrial machinery".into() },
        CpvEntry { code: "43000000".into(), description: "Machinery for mining, quarrying, construction equipment".into() },
        CpvEntry { code: "44000000".into(), description: "Construction structures and materials; auxiliary products to construction (except electric apparatus)".into() },
        CpvEntry { code: "45000000".into(), description: "Construction work".into() },
        CpvEntry { code: "48000000".into(), description: "Software package and information systems".into() },
        CpvEntry { code: "50000000".into(), description: "Repair and maintenance services".into() },
        CpvEntry { code: "51000000".into(), description: "Installation services (except software)".into() },
        CpvEntry { code: "55000000".into(), description: "Hotel, restaurant and retail trade services".into() },
        CpvEntry { code: "60000000".into(), description: "Transport services (excl. waste transport)".into() },
        CpvEntry { code: "63000000".into(), description: "Supporting and auxiliary transport services; travel agencies services".into() },
        CpvEntry { code: "64000000".into(), description: "Postal and telecommunications services".into() },
        CpvEntry { code: "65000000".into(), description: "Public utilities".into() },
        CpvEntry { code: "66000000".into(), description: "Financial and insurance services".into() },
        CpvEntry { code: "70000000".into(), description: "Real estate services".into() },
        CpvEntry { code: "71000000".into(), description: "Architectural, construction, engineering and inspection services".into() },
        CpvEntry { code: "72000000".into(), description: "IT services: consulting, software development, Internet and support".into() },
        CpvEntry { code: "73000000".into(), description: "Research and development services and related consultancy services".into() },
        CpvEntry { code: "75000000".into(), description: "Administration, defence and social security services".into() },
        CpvEntry { code: "76000000".into(), description: "Services related to the oil and gas industry".into() },
        CpvEntry { code: "77000000".into(), description: "Agricultural, forestry, horticultural, aquacultural and apicultural services".into() },
        CpvEntry { code: "79000000".into(), description: "Business services: law, marketing, consulting, recruitment, printing and security".into() },
        CpvEntry { code: "80000000".into(), description: "Education and training services".into() },
        CpvEntry { code: "85000000".into(), description: "Health and social work services".into() },
        CpvEntry { code: "90000000".into(), description: "Sewage, refuse, cleaning and environmental services".into() },
        CpvEntry { code: "92000000".into(), description: "Recreational, cultural and sporting services".into() },
        CpvEntry { code: "98000000".into(), description: "Other community, social and personal services".into() },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize CPV: {e}")))?;
    let path = dest_dir.join("cpv_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write CPV: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 13. Dewey Decimal Codes (embedded)
// ---------------------------------------------------------------------------

fn download_and_process_dewey(dest_dir: &Path) -> Result<u64, DatjitError> {
    let entries: Vec<DeweyEntry> = vec![
        DeweyEntry {
            code: "000".into(),
            description: "Computer science, information & general works".into(),
        },
        DeweyEntry {
            code: "010".into(),
            description: "Bibliographies".into(),
        },
        DeweyEntry {
            code: "020".into(),
            description: "Library & information sciences".into(),
        },
        DeweyEntry {
            code: "030".into(),
            description: "Encyclopedias & books of facts".into(),
        },
        DeweyEntry {
            code: "040".into(),
            description: "[Unassigned]".into(),
        },
        DeweyEntry {
            code: "050".into(),
            description: "Magazines, journals & serials".into(),
        },
        DeweyEntry {
            code: "060".into(),
            description: "Associations, organizations & museums".into(),
        },
        DeweyEntry {
            code: "070".into(),
            description: "News media, journalism & publishing".into(),
        },
        DeweyEntry {
            code: "080".into(),
            description: "Quotations".into(),
        },
        DeweyEntry {
            code: "090".into(),
            description: "Manuscripts & rare books".into(),
        },
        DeweyEntry {
            code: "100".into(),
            description: "Philosophy & psychology".into(),
        },
        DeweyEntry {
            code: "110".into(),
            description: "Metaphysics".into(),
        },
        DeweyEntry {
            code: "120".into(),
            description: "Epistemology".into(),
        },
        DeweyEntry {
            code: "130".into(),
            description: "Parapsychology & occultism".into(),
        },
        DeweyEntry {
            code: "140".into(),
            description: "Philosophical schools of thought".into(),
        },
        DeweyEntry {
            code: "150".into(),
            description: "Psychology".into(),
        },
        DeweyEntry {
            code: "160".into(),
            description: "Logic".into(),
        },
        DeweyEntry {
            code: "170".into(),
            description: "Ethics".into(),
        },
        DeweyEntry {
            code: "180".into(),
            description: "Ancient, medieval & eastern philosophy".into(),
        },
        DeweyEntry {
            code: "190".into(),
            description: "Modern western philosophy".into(),
        },
        DeweyEntry {
            code: "200".into(),
            description: "Religion".into(),
        },
        DeweyEntry {
            code: "210".into(),
            description: "Philosophy & theory of religion".into(),
        },
        DeweyEntry {
            code: "220".into(),
            description: "The Bible".into(),
        },
        DeweyEntry {
            code: "230".into(),
            description: "Christianity & Christian theology".into(),
        },
        DeweyEntry {
            code: "240".into(),
            description: "Christian practice & observance".into(),
        },
        DeweyEntry {
            code: "250".into(),
            description: "Christian pastoral practice & religious orders".into(),
        },
        DeweyEntry {
            code: "260".into(),
            description: "Christian organization, social work & worship".into(),
        },
        DeweyEntry {
            code: "270".into(),
            description: "History of Christianity".into(),
        },
        DeweyEntry {
            code: "280".into(),
            description: "Christian denominations".into(),
        },
        DeweyEntry {
            code: "290".into(),
            description: "Other religions".into(),
        },
        DeweyEntry {
            code: "300".into(),
            description: "Social sciences".into(),
        },
        DeweyEntry {
            code: "310".into(),
            description: "Statistics".into(),
        },
        DeweyEntry {
            code: "320".into(),
            description: "Political science".into(),
        },
        DeweyEntry {
            code: "330".into(),
            description: "Economics".into(),
        },
        DeweyEntry {
            code: "340".into(),
            description: "Law".into(),
        },
        DeweyEntry {
            code: "350".into(),
            description: "Public administration & military science".into(),
        },
        DeweyEntry {
            code: "360".into(),
            description: "Social problems & social services".into(),
        },
        DeweyEntry {
            code: "370".into(),
            description: "Education".into(),
        },
        DeweyEntry {
            code: "380".into(),
            description: "Commerce, communications & transportation".into(),
        },
        DeweyEntry {
            code: "390".into(),
            description: "Customs, etiquette & folklore".into(),
        },
        DeweyEntry {
            code: "400".into(),
            description: "Language".into(),
        },
        DeweyEntry {
            code: "410".into(),
            description: "Linguistics".into(),
        },
        DeweyEntry {
            code: "420".into(),
            description: "English & Old English languages".into(),
        },
        DeweyEntry {
            code: "430".into(),
            description: "German & related languages".into(),
        },
        DeweyEntry {
            code: "440".into(),
            description: "French & related languages".into(),
        },
        DeweyEntry {
            code: "450".into(),
            description: "Italian, Romanian & related languages".into(),
        },
        DeweyEntry {
            code: "460".into(),
            description: "Spanish & Portuguese languages".into(),
        },
        DeweyEntry {
            code: "470".into(),
            description: "Latin & Italic languages".into(),
        },
        DeweyEntry {
            code: "480".into(),
            description: "Classical & modern Greek languages".into(),
        },
        DeweyEntry {
            code: "490".into(),
            description: "Other languages".into(),
        },
        DeweyEntry {
            code: "500".into(),
            description: "Science".into(),
        },
        DeweyEntry {
            code: "510".into(),
            description: "Mathematics".into(),
        },
        DeweyEntry {
            code: "520".into(),
            description: "Astronomy".into(),
        },
        DeweyEntry {
            code: "530".into(),
            description: "Physics".into(),
        },
        DeweyEntry {
            code: "540".into(),
            description: "Chemistry".into(),
        },
        DeweyEntry {
            code: "550".into(),
            description: "Earth sciences & geology".into(),
        },
        DeweyEntry {
            code: "560".into(),
            description: "Fossils & prehistoric life".into(),
        },
        DeweyEntry {
            code: "570".into(),
            description: "Life sciences; biology".into(),
        },
        DeweyEntry {
            code: "580".into(),
            description: "Plants (Botany)".into(),
        },
        DeweyEntry {
            code: "590".into(),
            description: "Animals (Zoology)".into(),
        },
        DeweyEntry {
            code: "600".into(),
            description: "Technology".into(),
        },
        DeweyEntry {
            code: "610".into(),
            description: "Medicine & health".into(),
        },
        DeweyEntry {
            code: "620".into(),
            description: "Engineering".into(),
        },
        DeweyEntry {
            code: "630".into(),
            description: "Agriculture".into(),
        },
        DeweyEntry {
            code: "640".into(),
            description: "Home & family management".into(),
        },
        DeweyEntry {
            code: "650".into(),
            description: "Management & public relations".into(),
        },
        DeweyEntry {
            code: "660".into(),
            description: "Chemical engineering".into(),
        },
        DeweyEntry {
            code: "670".into(),
            description: "Manufacturing".into(),
        },
        DeweyEntry {
            code: "680".into(),
            description: "Manufacture for specific uses".into(),
        },
        DeweyEntry {
            code: "690".into(),
            description: "Building & construction".into(),
        },
        DeweyEntry {
            code: "700".into(),
            description: "Arts".into(),
        },
        DeweyEntry {
            code: "710".into(),
            description: "Landscaping & area planning".into(),
        },
        DeweyEntry {
            code: "720".into(),
            description: "Architecture".into(),
        },
        DeweyEntry {
            code: "730".into(),
            description: "Sculpture, ceramics & metalwork".into(),
        },
        DeweyEntry {
            code: "740".into(),
            description: "Drawing & decorative arts".into(),
        },
        DeweyEntry {
            code: "750".into(),
            description: "Painting".into(),
        },
        DeweyEntry {
            code: "760".into(),
            description: "Graphic arts".into(),
        },
        DeweyEntry {
            code: "770".into(),
            description: "Photography & computer art".into(),
        },
        DeweyEntry {
            code: "780".into(),
            description: "Music".into(),
        },
        DeweyEntry {
            code: "790".into(),
            description: "Sports, games & entertainment".into(),
        },
        DeweyEntry {
            code: "800".into(),
            description: "Literature".into(),
        },
        DeweyEntry {
            code: "810".into(),
            description: "American literature in English".into(),
        },
        DeweyEntry {
            code: "820".into(),
            description: "English & Old English literatures".into(),
        },
        DeweyEntry {
            code: "830".into(),
            description: "German & related literatures".into(),
        },
        DeweyEntry {
            code: "840".into(),
            description: "French & related literatures".into(),
        },
        DeweyEntry {
            code: "850".into(),
            description: "Italian, Romanian & related literatures".into(),
        },
        DeweyEntry {
            code: "860".into(),
            description: "Spanish & Portuguese literatures".into(),
        },
        DeweyEntry {
            code: "870".into(),
            description: "Latin & Italic literatures".into(),
        },
        DeweyEntry {
            code: "880".into(),
            description: "Classical & modern Greek literatures".into(),
        },
        DeweyEntry {
            code: "890".into(),
            description: "Other literatures".into(),
        },
        DeweyEntry {
            code: "900".into(),
            description: "History & geography".into(),
        },
        DeweyEntry {
            code: "910".into(),
            description: "Geography & travel".into(),
        },
        DeweyEntry {
            code: "920".into(),
            description: "Biography & genealogy".into(),
        },
        DeweyEntry {
            code: "930".into(),
            description: "History of ancient world (to ca. 499)".into(),
        },
        DeweyEntry {
            code: "940".into(),
            description: "History of Europe".into(),
        },
        DeweyEntry {
            code: "950".into(),
            description: "History of Asia".into(),
        },
        DeweyEntry {
            code: "960".into(),
            description: "History of Africa".into(),
        },
        DeweyEntry {
            code: "970".into(),
            description: "History of North America".into(),
        },
        DeweyEntry {
            code: "980".into(),
            description: "History of South America".into(),
        },
        DeweyEntry {
            code: "990".into(),
            description: "History of other areas".into(),
        },
    ];

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize Dewey: {e}")))?;
    let path = dest_dir.join("dewey_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write Dewey: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taxonomy_known_sources_count() {
        let sources = taxonomy_known_sources();
        assert_eq!(sources.len(), 13);
    }

    #[test]
    fn test_taxonomy_known_sources_categories() {
        let sources = taxonomy_known_sources();
        for source in &sources {
            assert_eq!(source.category, "taxonomy");
        }
    }

    #[test]
    fn test_taxonomy_known_sources_names() {
        let sources = taxonomy_known_sources();
        let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"NAICS Industry Codes"));
        assert!(names.contains(&"ISIC Rev.4 Codes"));
        assert!(names.contains(&"SIC Codes"));
        assert!(names.contains(&"UNSPSC Codes"));
        assert!(names.contains(&"HS Codes"));
        assert!(names.contains(&"MCC Codes"));
        assert!(names.contains(&"CIP Codes"));
        assert!(names.contains(&"ISCED Codes"));
        assert!(names.contains(&"ELF Codes"));
        assert!(names.contains(&"LEI Registry"));
        assert!(names.contains(&"COFOG Codes"));
        assert!(names.contains(&"CPV Codes"));
        assert!(names.contains(&"Dewey Decimal Codes"));
    }

    #[test]
    fn test_taxonomy_known_sources_licenses() {
        let sources = taxonomy_known_sources();
        for source in &sources {
            assert!(
                !source.license.is_empty(),
                "source {} has no license",
                source.name
            );
        }
    }

    #[test]
    fn test_embedded_unspsc() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_and_process_unspsc(dir.path());
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);
        let path = dir.path().join("unspsc_codes.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let entries: Vec<UnspscEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 56);
    }

    #[test]
    fn test_embedded_isced() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_and_process_isced(dir.path());
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);
        let path = dir.path().join("isced_codes.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let entries: Vec<IscedEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 46);
    }

    #[test]
    fn test_embedded_dewey() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_and_process_dewey(dir.path());
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);
        let path = dir.path().join("dewey_codes.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let entries: Vec<DeweyEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 100);
    }
}
