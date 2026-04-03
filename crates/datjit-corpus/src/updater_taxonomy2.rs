//! Batch 8: Additional taxonomy and classification corpus sources.
//!
//! This module provides corpus sources for: ATC drug codes, MeSH medical terms,
//! UN M.49 regions, ISO 3166-2 subdivisions, SWIFT/BIC codes (embedded),
//! NCBI taxonomy species, and IPCC emission categories (embedded).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read as IoRead;
use std::path::Path;

use datjit_core::error::DatjitError;
use serde::{Deserialize, Serialize};

use crate::updater::{download, download_source, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcEntry {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEntry {
    pub ui: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnM49Entry {
    pub code: String,
    pub name: String,
    pub region: String,
    pub sub_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iso31662Entry {
    pub code: String,
    pub name: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftBicEntry {
    pub bic: String,
    pub bank_name: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcbiTaxonEntry {
    pub taxon_id: u64,
    pub scientific_name: String,
    pub rank: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpccCategoryEntry {
    pub code: String,
    pub description: String,
    pub sector: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

/// Return the taxonomy2 corpus sources (Batch 8).
pub fn taxonomy2_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "ATC Codes".into(),
            description: "WHO Anatomical Therapeutic Chemical classification codes".into(),
            url: "https://raw.githubusercontent.com/fabkury/atcd/master/WHO%20ATC-DDD%202021-12-03.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "MeSH Terms".into(),
            description: "NLM Medical Subject Headings vocabulary".into(),
            url: "https://nlmpubs.nlm.nih.gov/projects/mesh/MESH_FILES/asciimesh/d2025.bin".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "UN M.49 Regions".into(),
            description: "UN M.49 standard country/region codes".into(),
            url: "https://raw.githubusercontent.com/lukes/ISO-3166-Countries-with-Regional-Codes/master/all/all.csv".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "ISO 3166-2 Subdivisions".into(),
            description: "ISO 3166-2 country subdivision codes".into(),
            url: "https://raw.githubusercontent.com/olahol/iso-3166-2.json/master/iso-3166-2.json".into(),
            license: "MIT".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "SWIFT/BIC Codes".into(),
            description: "Major world bank SWIFT/BIC identifiers (embedded)".into(),
            url: "embedded".into(),
            license: "Public Domain (embedded data)".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "NCBI Taxonomy".into(),
            description: "NCBI taxonomy species and genus names".into(),
            url: "https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdmp.zip".into(),
            license: "Public Domain".into(),
            category: "taxonomy".into(),
        },
        CorpusSource {
            name: "IPCC Emission Categories".into(),
            description: "IPCC greenhouse gas emission source categories (embedded)".into(),
            url: "embedded".into(),
            license: "Public Domain (embedded data)".into(),
            category: "taxonomy".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Download all Batch 8 taxonomy corpus sources and write results into the temp directories.
pub fn download_taxonomy2_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "ATC Codes",
        "shared/atc_codes.json",
        || download_and_process_atc(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "MeSH Terms",
        "shared/mesh_terms.json",
        || download_and_process_mesh(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "UN M.49 Regions",
        "shared/un_m49.json",
        || download_and_process_un_m49(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "ISO 3166-2 Subdivisions",
        "shared/iso_3166_2.json",
        || download_and_process_iso_3166_2(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "SWIFT/BIC Codes",
        "shared/swift_bic.json",
        || download_and_process_swift_bic(temp_shared),
        report,
        on_progress,
    );

    download_source(
        "NCBI Taxonomy",
        "shared/ncbi_taxonomy.json",
        || download_and_process_ncbi_taxonomy(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "IPCC Emission Categories",
        "shared/ipcc_categories.json",
        || download_and_process_ipcc(temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// 1. ATC Codes (drug classification)
// ---------------------------------------------------------------------------

fn download_and_process_atc(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/fabkury/atcd/master/WHO%20ATC-DDD%202021-12-03.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read ATC headers: {e}")))?
        .clone();

    // Find code and name columns flexibly
    let code_idx = headers
        .iter()
        .position(|h| {
            let low = h.to_lowercase();
            low.contains("atc_code") || low.contains("atc code") || low == "code"
        })
        .unwrap_or(0);
    let name_idx = headers
        .iter()
        .position(|h| {
            let low = h.to_lowercase();
            low.contains("atc_name") || low.contains("atc name") || low.contains("name")
        })
        .unwrap_or(1);

    let mut seen = HashSet::new();
    let mut entries: Vec<AtcEntry> = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        if code.is_empty() || name.is_empty() {
            continue;
        }
        if !seen.insert(code.clone()) {
            continue;
        }
        entries.push(AtcEntry { code, name });
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ATC codes: {e}")))?;
    let path = dest_dir.join("atc_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ATC codes: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 2. MeSH Terms (medical subject headings)
// ---------------------------------------------------------------------------

fn download_and_process_mesh(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://nlmpubs.nlm.nih.gov/projects/mesh/MESH_FILES/asciimesh/d2025.bin",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<MeshEntry> = Vec::new();
    let mut current_mh: Option<String> = None;
    let mut current_ui: Option<String> = None;

    for line in text.lines() {
        if line.starts_with("*NEWRECORD") {
            // Emit previous record if complete
            if let (Some(ui), Some(name)) = (current_ui.take(), current_mh.take()) {
                entries.push(MeshEntry { ui, name });
                if entries.len() >= 10_000 {
                    break;
                }
            }
            current_mh = None;
            current_ui = None;
            continue;
        }

        if let Some(rest) = line.strip_prefix("MH = ") {
            current_mh = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("UI = ") {
            current_ui = Some(rest.trim().to_string());
        }
    }

    // Don't forget the last record
    if entries.len() < 10_000 {
        if let (Some(ui), Some(name)) = (current_ui, current_mh) {
            entries.push(MeshEntry { ui, name });
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no MeSH entries parsed from d2025.bin".into(),
        ));
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize MeSH terms: {e}")))?;
    let path = dest_dir.join("mesh_terms.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write MeSH terms: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 3. UN M.49 (regions/countries with regional codes)
// ---------------------------------------------------------------------------

fn download_and_process_un_m49(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/lukes/ISO-3166-Countries-with-Regional-Codes/master/all/all.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read UN M.49 headers: {e}")))?
        .clone();

    let name_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "name")
        .unwrap_or(0);
    let code_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "country-code")
        .unwrap_or(3);
    let region_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "region")
        .unwrap_or(5);
    let sub_region_idx = headers
        .iter()
        .position(|h| h.to_lowercase() == "sub-region")
        .unwrap_or(6);

    let mut entries: Vec<UnM49Entry> = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        let code = record.get(code_idx).unwrap_or("").trim().to_string();
        let region = record.get(region_idx).unwrap_or("").trim().to_string();
        let sub_region = record.get(sub_region_idx).unwrap_or("").trim().to_string();
        if name.is_empty() || code.is_empty() {
            continue;
        }
        entries.push(UnM49Entry {
            code,
            name,
            region,
            sub_region,
        });
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize UN M.49: {e}")))?;
    let path = dest_dir.join("un_m49.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write UN M.49: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 4. ISO 3166-2 (country subdivisions)
// ---------------------------------------------------------------------------

fn download_and_process_iso_3166_2(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/olahol/iso-3166-2.json/master/iso-3166-2.json",
    )?;

    // JSON format: { "US": { "US-CA": "California", ... }, ... }
    let map: HashMap<String, HashMap<String, String>> = serde_json::from_slice(&data)
        .map_err(|e| DatjitError::Corpus(format!("parse ISO 3166-2 JSON: {e}")))?;

    let mut entries: Vec<Iso31662Entry> = Vec::new();

    for (country, subdivisions) in &map {
        for (code, name) in subdivisions {
            entries.push(Iso31662Entry {
                code: code.clone(),
                name: name.clone(),
                country: country.clone(),
            });
        }
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize ISO 3166-2: {e}")))?;
    let path = dest_dir.join("iso_3166_2.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write ISO 3166-2: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 5. SWIFT/BIC Codes (embedded)
// ---------------------------------------------------------------------------

fn download_and_process_swift_bic(dest_dir: &Path) -> Result<u64, DatjitError> {
    let raw_data: Vec<(&str, &str, &str)> = vec![
        ("BOFAUS3N", "Bank of America", "US"),
        ("CHASUS33", "JPMorgan Chase", "US"),
        ("CITIUS33", "Citibank", "US"),
        ("WFBIUS6S", "Wells Fargo", "US"),
        ("USBKUS44", "U.S. Bancorp", "US"),
        ("PNCCUS33", "PNC Financial", "US"),
        ("TRWIUS33", "Truist Financial", "US"),
        ("TABORNUS", "TD Bank", "US"),
        ("HATRUS44", "HSBC USA", "US"),
        ("FNBOUS44", "Fifth Third Bank", "US"),
        ("KEYBUS33", "KeyBank", "US"),
        ("MRMDUS33", "M&T Bank", "US"),
        ("BARCGB22", "Barclays", "GB"),
        ("HSBCGB2L", "HSBC Holdings", "GB"),
        ("LOYDGB2L", "Lloyds Banking Group", "GB"),
        ("NWBKGB2L", "NatWest Group", "GB"),
        ("SCBLGB2L", "Standard Chartered", "GB"),
        ("BKENGB2L", "Bank of England", "GB"),
        ("ABORAB22", "Virgin Money", "GB"),
        ("MIDLGB22", "HSBC UK", "GB"),
        ("BNPAFRPP", "BNP Paribas", "FR"),
        ("SOGEFRPP", "Societe Generale", "FR"),
        ("CRLYFRPP", "Credit Agricole", "FR"),
        ("CEPAFRPP", "Caisse d'Epargne", "FR"),
        ("CCFRFRPP", "HSBC France", "FR"),
        ("DEUTDEFF", "Deutsche Bank", "DE"),
        ("COBADEFF", "Commerzbank", "DE"),
        ("HYVEDEMM", "HypoVereinsbank", "DE"),
        ("BELADEBE", "Berliner Sparkasse", "DE"),
        ("DRESDEFF", "Dresdner Bank", "DE"),
        ("BSCHESMM", "Santander Spain", "ES"),
        ("CABOROBANK", "CaixaBank", "ES"),
        ("BBVAESMM", "BBVA", "ES"),
        ("SABLESBB", "Banco Sabadell", "ES"),
        ("CAABOROIG", "Bankinter", "ES"),
        ("BCITITMM", "Intesa Sanpaolo", "IT"),
        ("UNCRITMM", "UniCredit", "IT"),
        ("BPMOIT22", "Banco BPM", "IT"),
        ("POSOIT22", "Poste Italiane", "IT"),
        ("BAPPIT22", "Banca Popolare", "IT"),
        ("RBOSGB2L", "Royal Bank of Scotland", "GB"),
        ("ABNANL2A", "ABN AMRO", "NL"),
        ("INGBNL2A", "ING Group", "NL"),
        ("RABONL2U", "Rabobank", "NL"),
        ("UBSWCHZH", "UBS", "CH"),
        ("CRESCHZZ", "Credit Suisse", "CH"),
        ("ZKBKCHZZ", "Zurcher Kantonalbank", "CH"),
        ("BOFCNBJS", "Bank of China", "CN"),
        ("ICBKCNBJ", "ICBC", "CN"),
        ("PCBCCNBJ", "China Construction Bank", "CN"),
        ("ABOCCNBJ", "Agricultural Bank of China", "CN"),
        ("COMMCNSH", "Bank of Communications", "CN"),
        ("MABORJPJT", "MUFG Bank", "JP"),
        ("SMBJJPJT", "Sumitomo Mitsui", "JP"),
        ("MHCBJPJT", "Mizuho Bank", "JP"),
        ("BOABORTKJPJT", "Resona Bank", "JP"),
        ("ANZBAU3M", "ANZ Bank", "AU"),
        ("CTBAAU2S", "Commonwealth Bank", "AU"),
        ("NABABQU2S", "National Australia Bank", "AU"),
        ("WPACAU2S", "Westpac", "AU"),
        ("BKCHCNBJ", "Bank of China HK", "HK"),
        ("HSBCHKHH", "HSBC Hong Kong", "HK"),
        ("SCBLSGSG", "Standard Chartered SG", "SG"),
        ("DBSSSGSG", "DBS Bank", "SG"),
        ("OCBCSGSG", "OCBC Bank", "SG"),
        ("UABORSBKRSE", "UOB", "SG"),
        ("CIABORABKA22", "Citibank Korea", "KR"),
        ("KOEXKRSE", "Korea Exchange Bank", "KR"),
        ("HABORANAKRSE", "Hana Bank", "KR"),
        ("BKIDJA", "Bank Indonesia", "ID"),
        ("BMRIIDJA", "Bank Mandiri", "ID"),
        ("BNINIDJIA", "BNI", "ID"),
        ("BRINIDJA", "BRI", "ID"),
        ("SBININBB", "State Bank of India", "IN"),
        ("HDFCINBB", "HDFC Bank", "IN"),
        ("ABORICIINBB", "ICICI Bank", "IN"),
        ("AXISINBB", "Axis Bank", "IN"),
        ("BMCEBRMM", "Bradesco", "BR"),
        ("ITAUBRSP", "Itau Unibanco", "BR"),
        ("BRASBRRJ", "Banco do Brasil", "BR"),
        ("ABORBNKCDEFF", "Bundesbank", "DE"),
        ("SNORRUMM", "Sberbank", "RU"),
        ("ALFLRUMM", "Alfa-Bank", "RU"),
        ("TABORIMYKT", "Maybank", "MY"),
        ("CIBBMYKL", "CIMB", "MY"),
        ("BKKBTHBK", "Bangkok Bank", "TH"),
        ("SICOTHBK", "Siam Commercial Bank", "TH"),
        ("FNBOZAJJ", "First National Bank", "ZA"),
        ("SBZAZAJJ", "Standard Bank", "ZA"),
        ("ABSAZAJJ", "Absa", "ZA"),
        ("NEDSZAJJ", "Nedbank", "ZA"),
        ("TDOMCATT", "TD Canada Trust", "CA"),
        ("ROYCCAT2", "Royal Bank of Canada", "CA"),
        ("BNDCCAMM", "National Bank of Canada", "CA"),
        ("NOSCCATT", "Scotiabank", "CA"),
        ("CAFECHGG", "Chilean Central Bank", "CL"),
        ("BABORCLRMCL", "BancoEstado", "CL"),
        ("NACABORLARGEST", "Bancolombia", "CO"),
        ("COLOCOBM", "Banco de Colombia", "CO"),
        ("BAMABOROMXMM", "BBVA Mexico", "MX"),
        ("BIMEMXMM", "Banamex", "MX"),
    ];

    let entries: Vec<SwiftBicEntry> = raw_data
        .into_iter()
        .map(|(bic, bank_name, country)| SwiftBicEntry {
            bic: bic.to_string(),
            bank_name: bank_name.to_string(),
            country: country.to_string(),
        })
        .collect();

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize SWIFT/BIC codes: {e}")))?;
    let path = dest_dir.join("swift_bic.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write SWIFT/BIC codes: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 6. NCBI Taxonomy (species)
// ---------------------------------------------------------------------------

fn download_and_process_ncbi_taxonomy(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdmp.zip",
    )?;

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("open NCBI taxonomy zip: {e}")))?;

    // Parse nodes.dmp to get taxids with rank "species" or "genus"
    let mut valid_taxids: HashMap<u64, String> = HashMap::new();
    {
        let mut nodes_file = archive
            .by_name("nodes.dmp")
            .map_err(|e| DatjitError::Corpus(format!("extract nodes.dmp: {e}")))?;
        let mut nodes_content = String::new();
        nodes_file
            .read_to_string(&mut nodes_content)
            .map_err(|e| DatjitError::Corpus(format!("read nodes.dmp: {e}")))?;

        for line in nodes_content.lines() {
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() < 3 {
                continue;
            }
            let taxid_str = fields[0].trim().trim_end_matches("\t|");
            let rank = fields[2].trim().trim_end_matches("\t|");
            if rank == "species" || rank == "genus" {
                if let Ok(taxid) = taxid_str.parse::<u64>() {
                    valid_taxids.insert(taxid, rank.to_string());
                }
            }
        }
    }

    // Parse names.dmp to get scientific names for those taxids
    let mut entries: Vec<NcbiTaxonEntry> = Vec::new();
    {
        let mut names_file = archive
            .by_name("names.dmp")
            .map_err(|e| DatjitError::Corpus(format!("extract names.dmp: {e}")))?;
        let mut names_content = String::new();
        names_file
            .read_to_string(&mut names_content)
            .map_err(|e| DatjitError::Corpus(format!("read names.dmp: {e}")))?;

        for line in names_content.lines() {
            if entries.len() >= 10_000 {
                break;
            }
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() < 4 {
                continue;
            }
            let name_class = fields[3].trim().trim_end_matches("\t|");
            if name_class != "scientific name" {
                continue;
            }
            let taxid_str = fields[0].trim().trim_end_matches("\t|");
            let taxid = match taxid_str.parse::<u64>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            if let Some(rank) = valid_taxids.get(&taxid) {
                let scientific_name = fields[1].trim().to_string();
                if scientific_name.is_empty() {
                    continue;
                }
                entries.push(NcbiTaxonEntry {
                    taxon_id: taxid,
                    scientific_name,
                    rank: rank.clone(),
                });
            }
        }
    }

    if entries.is_empty() {
        return Err(DatjitError::Corpus(
            "no NCBI taxonomy entries parsed".into(),
        ));
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize NCBI taxonomy: {e}")))?;
    let path = dest_dir.join("ncbi_taxonomy.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write NCBI taxonomy: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// 7. IPCC Emission Categories (embedded)
// ---------------------------------------------------------------------------

fn download_and_process_ipcc(dest_dir: &Path) -> Result<u64, DatjitError> {
    let raw_data: Vec<(&str, &str, &str)> = vec![
        ("1", "Energy", "Energy"),
        ("1.A", "Fuel Combustion Activities", "Energy"),
        ("1.A.1", "Energy Industries", "Energy"),
        (
            "1.A.2",
            "Manufacturing Industries and Construction",
            "Energy",
        ),
        ("1.A.3", "Transport", "Energy"),
        ("1.A.4", "Other Sectors", "Energy"),
        ("1.A.5", "Non-Specified", "Energy"),
        ("1.B", "Fugitive Emissions from Fuels", "Energy"),
        ("1.B.1", "Solid Fuels", "Energy"),
        ("1.B.2", "Oil and Natural Gas", "Energy"),
        ("1.B.3", "Other Emissions from Energy Production", "Energy"),
        ("1.C", "Carbon Dioxide Transport and Storage", "Energy"),
        ("2", "Industrial Processes and Product Use", "IPPU"),
        ("2.A", "Mineral Industry", "IPPU"),
        ("2.B", "Chemical Industry", "IPPU"),
        ("2.C", "Metal Industry", "IPPU"),
        (
            "2.D",
            "Non-Energy Products from Fuels and Solvent Use",
            "IPPU",
        ),
        ("2.E", "Electronics Industry", "IPPU"),
        ("2.F", "Product Uses as Substitutes for ODS", "IPPU"),
        ("2.G", "Other Product Manufacture and Use", "IPPU"),
        ("2.H", "Other", "IPPU"),
        ("3", "Agriculture, Forestry and Other Land Use", "AFOLU"),
        ("3.A", "Livestock", "AFOLU"),
        ("3.A.1", "Enteric Fermentation", "AFOLU"),
        ("3.A.2", "Manure Management", "AFOLU"),
        ("3.B", "Land", "AFOLU"),
        ("3.B.1", "Forest Land", "AFOLU"),
        ("3.B.2", "Cropland", "AFOLU"),
        ("3.B.3", "Grassland", "AFOLU"),
        ("3.B.4", "Wetlands", "AFOLU"),
        ("3.B.5", "Settlements", "AFOLU"),
        ("3.B.6", "Other Land", "AFOLU"),
        (
            "3.C",
            "Aggregate Sources and Non-CO2 Emissions on Land",
            "AFOLU",
        ),
        ("3.C.1", "Emissions from Biomass Burning", "AFOLU"),
        ("3.C.2", "Liming", "AFOLU"),
        ("3.C.3", "Urea Application", "AFOLU"),
        ("3.C.4", "Direct N2O Emissions from Managed Soils", "AFOLU"),
        (
            "3.C.5",
            "Indirect N2O Emissions from Managed Soils",
            "AFOLU",
        ),
        (
            "3.C.6",
            "Indirect N2O Emissions from Manure Management",
            "AFOLU",
        ),
        ("3.C.7", "Rice Cultivations", "AFOLU"),
        ("3.D", "Other", "AFOLU"),
        ("4", "Waste", "Waste"),
        ("4.A", "Solid Waste Disposal", "Waste"),
        ("4.B", "Biological Treatment of Solid Waste", "Waste"),
        ("4.C", "Incineration and Open Burning of Waste", "Waste"),
        ("4.D", "Wastewater Treatment and Discharge", "Waste"),
        ("4.E", "Other", "Waste"),
        ("5", "Other", "Other"),
        (
            "5.A",
            "Indirect N2O Emissions from Atmospheric Deposition of Nitrogen in NOx and NH3",
            "Other",
        ),
        ("5.B", "Other", "Other"),
    ];

    let entries: Vec<IpccCategoryEntry> = raw_data
        .into_iter()
        .map(|(code, description, sector)| IpccCategoryEntry {
            code: code.to_string(),
            description: description.to_string(),
            sector: sector.to_string(),
        })
        .collect();

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize IPCC categories: {e}")))?;
    let path = dest_dir.join("ipcc_categories.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write IPCC categories: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taxonomy2_known_sources_count() {
        let sources = taxonomy2_known_sources();
        assert_eq!(sources.len(), 7);
    }

    #[test]
    fn test_taxonomy2_known_sources_categories() {
        let sources = taxonomy2_known_sources();
        for source in &sources {
            assert_eq!(source.category, "taxonomy");
        }
    }

    #[test]
    fn test_taxonomy2_known_sources_names() {
        let sources = taxonomy2_known_sources();
        let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"ATC Codes"));
        assert!(names.contains(&"MeSH Terms"));
        assert!(names.contains(&"UN M.49 Regions"));
        assert!(names.contains(&"ISO 3166-2 Subdivisions"));
        assert!(names.contains(&"SWIFT/BIC Codes"));
        assert!(names.contains(&"NCBI Taxonomy"));
        assert!(names.contains(&"IPCC Emission Categories"));
    }

    #[test]
    fn test_taxonomy2_known_sources_licenses() {
        let sources = taxonomy2_known_sources();
        for source in &sources {
            assert!(
                !source.license.is_empty(),
                "source {} has no license",
                source.name
            );
        }
    }

    #[test]
    fn test_swift_bic_embedded() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_and_process_swift_bic(dir.path());
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);

        let content = fs::read_to_string(dir.path().join("swift_bic.json")).unwrap();
        let entries: Vec<SwiftBicEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 101);
        // Check a known entry
        assert!(entries.iter().any(|e| e.bic == "CHASUS33"));
    }

    #[test]
    fn test_ipcc_embedded() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_and_process_ipcc(dir.path());
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);

        let content = fs::read_to_string(dir.path().join("ipcc_categories.json")).unwrap();
        let entries: Vec<IpccCategoryEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 50);
        // Check first and last
        assert_eq!(entries[0].code, "1");
        assert_eq!(entries[0].sector, "Energy");
        assert_eq!(entries.last().unwrap().code, "5.B");
    }
}
