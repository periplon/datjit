use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use datjit_core::error::DatjitError;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

/// A single corpus entry with a name and optional weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    pub name: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
}

fn default_weight() -> f64 {
    1.0
}

/// Report of what was updated during a corpus update operation.
pub struct CorpusUpdateReport {
    pub files_updated: Vec<String>,
    pub files_failed: Vec<(String, String)>,
    pub total_size_bytes: u64,
}

/// Known corpus sources with their metadata.
pub struct CorpusSource {
    pub name: String,
    pub description: String,
    pub url: String,
    pub license: String,
    pub category: String,
}

/// Status of the local corpus installation.
pub struct CorpusStatus {
    pub corpus_dir: PathBuf,
    pub installed_locales: Vec<String>,
    pub installed_files: Vec<(String, u64)>,
    pub total_size_bytes: u64,
}

/// List all known corpus sources.
pub fn known_sources() -> Vec<CorpusSource> {
    let mut sources = vec![
        CorpusSource {
            name: "US Census First Names".into(),
            description: "US Census Bureau 1990 first name frequency data".into(),
            url: "https://www2.census.gov/topics/genealogy/1990surnames/".into(),
            license: "Public Domain".into(),
            category: "person".into(),
        },
        CorpusSource {
            name: "US Census Surnames".into(),
            description: "US Census Bureau surname frequency data".into(),
            url: "https://www2.census.gov/topics/genealogy/2010surnames/names.zip".into(),
            license: "Public Domain".into(),
            category: "person".into(),
        },
        CorpusSource {
            name: "GeoNames Cities".into(),
            description: "GeoNames geographical database of cities worldwide".into(),
            url: "https://download.geonames.org/export/dump/cities15000.zip".into(),
            license: "CC BY 4.0".into(),
            category: "address".into(),
        },
        CorpusSource {
            name: "O*NET Job Titles".into(),
            description: "O*NET occupational titles, alternate titles, and descriptions".into(),
            url: "https://www.onetcenter.org/dl_files/database/db_28_3_text/Occupation%20Data.txt"
                .into(),
            license: "CC BY 4.0".into(),
            category: "job".into(),
        },
        CorpusSource {
            name: "Faker.js en Names".into(),
            description: "Faker.js English first/last name lists (MIT)".into(),
            url: "https://raw.githubusercontent.com/faker-js/faker/main/src/locales/en/person/"
                .into(),
            license: "MIT".into(),
            category: "person".into(),
        },
        CorpusSource {
            name: "IANA Timezones".into(),
            description: "IANA time zone database zone list".into(),
            url: "https://data.iana.org/time-zones/tzdata-latest.tar.gz".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "GeoNames Countries".into(),
            description: "GeoNames country information database".into(),
            url: "https://download.geonames.org/export/dump/countryInfo.txt".into(),
            license: "CC BY 4.0".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "GeoNames Admin1 Regions".into(),
            description: "GeoNames first-level administrative divisions".into(),
            url: "https://download.geonames.org/export/dump/admin1CodesASCII.txt".into(),
            license: "CC BY 4.0".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "US Postal Codes".into(),
            description: "GeoNames US postal/ZIP code database".into(),
            url: "https://download.geonames.org/export/zip/US.zip".into(),
            license: "CC BY 4.0".into(),
            category: "address".into(),
        },
        CorpusSource {
            name: "Google Product Categories".into(),
            description: "Google product taxonomy with IDs".into(),
            url: "https://www.google.com/basepages/producttype/taxonomy-with-ids.en-US.txt".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Phone Formats".into(),
            description: "Google libphonenumber phone number metadata".into(),
            url: "https://raw.githubusercontent.com/google/libphonenumber/master/resources/PhoneNumberMetadata.xml".into(),
            license: "Apache 2.0".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "Credit Card BINs".into(),
            description: "Bank Identification Number list for major card brands".into(),
            url: "https://raw.githubusercontent.com/iannuttall/binlist-data/master/binlist-data.csv".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "O*NET Job Zones".into(),
            description: "O*NET job zone classifications for occupations".into(),
            url: "https://www.onetcenter.org/dl_files/database/db_28_3_text/Job%20Zones.txt".into(),
            license: "CC BY 4.0".into(),
            category: "job".into(),
        },
        CorpusSource {
            name: "CLDR Currencies".into(),
            description: "Unicode CLDR currency codes, names, symbols, and decimal digits".into(),
            url: "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/currencyData.json".into(),
            license: "Unicode License".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "ISO 639 Languages".into(),
            description: "ISO 639 language codes and names".into(),
            url: "https://www.loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "IANA TLDs".into(),
            description: "IANA top-level domain names".into(),
            url: "https://data.iana.org/TLD/tlds-alpha-by-domain.txt".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "IANA MIME Types".into(),
            description: "IANA registered media types (MIME types)".into(),
            url: "https://www.iana.org/assignments/media-types/media-types.xml".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "OurAirports".into(),
            description: "OurAirports worldwide airport database".into(),
            url: "https://davidmegginson.github.io/ourairports-data/airports.csv".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "OpenFlights Airlines".into(),
            description: "OpenFlights active airline database".into(),
            url: "https://raw.githubusercontent.com/jpatokal/openflights/master/data/airlines.dat".into(),
            license: "Open Database License".into(),
            category: "shared".into(),
        },
        CorpusSource {
            name: "EPA Vehicles".into(),
            description: "EPA fuel economy vehicle data".into(),
            url: "https://www.fueleconomy.gov/feg/epadata/vehicles.csv.zip".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
    ];
    sources.extend(crate::updater_extra::extra_known_sources());
    sources.extend(crate::updater_github::github_known_sources());
    sources
}

/// Get the default corpus directory (~/.datjit/corpus/).
pub fn default_corpus_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".datjit").join("corpus")
    } else {
        PathBuf::from(".datjit").join("corpus")
    }
}

/// Check corpus status: what's installed, what's available.
pub fn check_corpus_status() -> Result<CorpusStatus, DatjitError> {
    let corpus_dir = default_corpus_dir();

    if !corpus_dir.exists() {
        return Ok(CorpusStatus {
            corpus_dir,
            installed_locales: Vec::new(),
            installed_files: Vec::new(),
            total_size_bytes: 0,
        });
    }

    let mut installed_locales = Vec::new();
    let mut installed_files = Vec::new();
    let mut total_size_bytes = 0u64;

    let entries = fs::read_dir(&corpus_dir).map_err(DatjitError::Io)?;

    for entry in entries {
        let entry = entry.map_err(DatjitError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                installed_locales.push(name.to_string());
            }
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub in sub_entries.flatten() {
                    if sub.path().is_file() {
                        if let Ok(meta) = sub.metadata() {
                            let size = meta.len();
                            let fname = sub
                                .path()
                                .strip_prefix(&corpus_dir)
                                .unwrap_or(&sub.path())
                                .to_string_lossy()
                                .to_string();
                            installed_files.push((fname, size));
                            total_size_bytes += size;
                        }
                    }
                }
            }
        } else if path.is_file() {
            if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                let fname = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                installed_files.push((fname, size));
                total_size_bytes += size;
            }
        }
    }

    installed_locales.sort();

    Ok(CorpusStatus {
        corpus_dir,
        installed_locales,
        installed_files,
        total_size_bytes,
    })
}

/// Download and install all corpus data transactionally.
/// Downloads to a temp directory first, processes files, then moves into place.
pub fn update_corpus(
    corpus_dir: &Path,
    on_progress: &dyn Fn(&str),
) -> Result<CorpusUpdateReport, DatjitError> {
    let locale_dir = corpus_dir.join("en-US");
    let shared_dir = corpus_dir.join("shared");
    let temp_dir =
        tempfile::tempdir().map_err(|e| DatjitError::Corpus(format!("temp dir: {e}")))?;
    let temp_locale = temp_dir.path().join("en-US");
    let temp_shared = temp_dir.path().join("shared");
    fs::create_dir_all(&temp_locale)
        .map_err(|e| DatjitError::Corpus(format!("create temp locale dir: {e}")))?;
    fs::create_dir_all(&temp_shared)
        .map_err(|e| DatjitError::Corpus(format!("create temp shared dir: {e}")))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("datjit/0.1.0")
        .build()
        .map_err(|e| DatjitError::Corpus(format!("http client: {e}")))?;

    let mut report = CorpusUpdateReport {
        files_updated: Vec::new(),
        files_failed: Vec::new(),
        total_size_bytes: 0,
    };

    // 1. Census First Names
    on_progress("Downloading US Census First Names...");
    match download_and_process_census_first(&client, &temp_locale) {
        Ok(size) => {
            report.files_updated.push("en-US/person_first.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  person_first.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("US Census First Names".into(), msg));
        }
    }

    // 2. US Census Surnames
    on_progress("Downloading US Census Surnames...");
    match download_and_process_census(&client, &temp_locale) {
        Ok(size) => {
            report.files_updated.push("en-US/person_last.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  person_last.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("US Census Surnames".into(), msg));
        }
    }

    // 3. GeoNames Cities
    on_progress("Downloading GeoNames Cities...");
    match download_and_process_geonames(&client, &temp_locale) {
        Ok(size) => {
            report.files_updated.push("en-US/cities.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  cities.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("GeoNames Cities".into(), msg));
        }
    }

    // 4. O*NET Job Titles
    on_progress("Downloading O*NET Job Titles...");
    match download_and_process_onet(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/job_titles.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/job_titles.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("O*NET Job Titles".into(), msg));
        }
    }

    // 5. IANA Timezones
    on_progress("Downloading IANA Timezones...");
    match download_and_process_timezones(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/timezones.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/timezones.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("IANA Timezones".into(), msg));
        }
    }

    // 6. GeoNames Countries
    on_progress("Downloading GeoNames Countries...");
    match download_and_process_countries(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/countries.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/countries.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("GeoNames Countries".into(), msg));
        }
    }

    // 7. GeoNames Admin1 Regions
    on_progress("Downloading GeoNames Admin1 Regions...");
    match download_and_process_admin1(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/admin1.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/admin1.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("GeoNames Admin1 Regions".into(), msg));
        }
    }

    // 8. US Postal Codes
    on_progress("Downloading US Postal Codes...");
    match download_and_process_postal_codes(&client, &temp_locale) {
        Ok(size) => {
            report.files_updated.push("en-US/postal_codes.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  en-US/postal_codes.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("US Postal Codes".into(), msg));
        }
    }

    // 9. Google Product Categories
    on_progress("Downloading Google Product Categories...");
    match download_and_process_product_categories(&client, &temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/product_categories.json".into());
            report.total_size_bytes += size;
            on_progress(&format!(
                "  shared/product_categories.json ({} KB)",
                size / 1024
            ));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("Google Product Categories".into(), msg));
        }
    }

    // 10. Phone Formats
    on_progress("Downloading Phone Formats...");
    match download_and_process_phone_formats(&client, &temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/phone_formats.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/phone_formats.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("Phone Formats".into(), msg));
        }
    }

    // 11. Credit Card BINs
    on_progress("Downloading Credit Card BINs...");
    match download_and_process_credit_card_bins(&client, &temp_shared) {
        Ok(size) => {
            report
                .files_updated
                .push("shared/credit_card_bins.json".into());
            report.total_size_bytes += size;
            on_progress(&format!(
                "  shared/credit_card_bins.json ({} KB)",
                size / 1024
            ));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("Credit Card BINs".into(), msg));
        }
    }

    // 12. CLDR Currencies
    on_progress("Downloading CLDR Currencies...");
    match download_and_process_currencies(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/currencies.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/currencies.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("CLDR Currencies".into(), msg));
        }
    }

    // 13. ISO 639 Languages
    on_progress("Downloading ISO 639 Languages...");
    match download_and_process_languages(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/languages.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/languages.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("ISO 639 Languages".into(), msg));
        }
    }

    // 14. IANA TLDs
    on_progress("Downloading IANA TLDs...");
    match download_and_process_tlds(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/tlds.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/tlds.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("IANA TLDs".into(), msg));
        }
    }

    // 15. IANA MIME Types
    on_progress("Downloading IANA MIME Types...");
    match download_and_process_mime_types(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/mime_types.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/mime_types.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("IANA MIME Types".into(), msg));
        }
    }

    // 16. OurAirports
    on_progress("Downloading OurAirports...");
    match download_and_process_airports(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/airports.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/airports.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("OurAirports".into(), msg));
        }
    }

    // 17. OpenFlights Airlines
    on_progress("Downloading OpenFlights Airlines...");
    match download_and_process_airlines(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/airlines.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/airlines.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report
                .files_failed
                .push(("OpenFlights Airlines".into(), msg));
        }
    }

    // 18. EPA Vehicles
    on_progress("Downloading EPA Vehicles...");
    match download_and_process_vehicles(&client, &temp_shared) {
        Ok(size) => {
            report.files_updated.push("shared/vehicles.json".into());
            report.total_size_bytes += size;
            on_progress(&format!("  shared/vehicles.json ({} KB)", size / 1024));
        }
        Err(e) => {
            let msg = format!("{e}");
            on_progress(&format!("  FAILED: {msg}"));
            report.files_failed.push(("EPA Vehicles".into(), msg));
        }
    }

    // Extra sources (Batch 3: domain-specific)
    crate::updater_extra::download_extra_sources(
        &client,
        &temp_shared,
        &temp_locale,
        &mut report,
        on_progress,
    );

    // GitHub sources (Batch 4: open datasets)
    crate::updater_github::download_github_sources(
        &client,
        &temp_shared,
        &temp_locale,
        &mut report,
        on_progress,
    );

    // If at least one file succeeded, move into place transactionally
    if !report.files_updated.is_empty() {
        fs::create_dir_all(&locale_dir)
            .map_err(|e| DatjitError::Corpus(format!("create corpus dir: {e}")))?;
        fs::create_dir_all(&shared_dir)
            .map_err(|e| DatjitError::Corpus(format!("create shared dir: {e}")))?;

        for fname in &report.files_updated {
            let src = temp_dir.path().join(fname);
            let dst = corpus_dir.join(fname);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(&src, &dst)
                .map_err(|e| DatjitError::Corpus(format!("move {fname} to corpus dir: {e}")))?;
        }

        on_progress(&format!(
            "Installed {} files to {}",
            report.files_updated.len(),
            corpus_dir.display()
        ));
    }

    if report.files_updated.is_empty() {
        return Err(DatjitError::Corpus(
            "all downloads failed — no files installed".into(),
        ));
    }

    Ok(report)
}

/// Download a URL and return the response bytes.
pub(crate) fn download(
    client: &reqwest::blocking::Client,
    url: &str,
) -> Result<Vec<u8>, DatjitError> {
    let resp = client
        .get(url)
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download {url}: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for {url}",
            resp.status()
        )));
    }

    resp.bytes()
        .map(|b| b.to_vec())
        .map_err(|e| DatjitError::Corpus(format!("read response {url}: {e}")))
}

/// Download Census 1990 first name frequency data and produce person_first.json.
/// Format: fixed-width text with name, frequency%, cumulative%, rank
fn download_and_process_census_first(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let mut entries: Vec<CorpusEntry> = Vec::new();

    // Download female first names
    let female_data = download(
        client,
        "https://www2.census.gov/topics/genealogy/1990surnames/dist.female.first",
    )?;
    for line in String::from_utf8_lossy(&female_data).lines() {
        if let Some(entry) = parse_census_name_line(line, "female") {
            entries.push(entry);
        }
    }

    // Download male first names
    let male_data = download(
        client,
        "https://www2.census.gov/topics/genealogy/1990surnames/dist.male.first",
    )?;
    for line in String::from_utf8_lossy(&male_data).lines() {
        if let Some(entry) = parse_census_name_line(line, "male") {
            entries.push(entry);
        }
    }

    // Sort by weight descending
    entries.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize first names: {e}")))?;
    let path = dest_dir.join("person_first.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write first names: {e}")))?;

    Ok(json.len() as u64)
}

/// Parse a Census 1990 name frequency line.
/// Format: "MARY           2.629  2.629      1" (name, freq%, cumfreq%, rank)
fn parse_census_name_line(line: &str, gender: &str) -> Option<CorpusEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let name = title_case(parts[0]);
        let weight: f64 = parts[1].parse().ok()?;
        Some(CorpusEntry {
            name,
            weight,
            gender: Some(gender.to_string()),
        })
    } else {
        None
    }
}

/// Download US Census surnames and produce person_last.json.
/// The zip contains Names_2010Census.csv with: name,rank,count,prop100k,...
fn download_and_process_census(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www2.census.gov/topics/genealogy/2010surnames/names.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip Census: {e}")))?;

    let mut entries: Vec<CorpusEntry> = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_lowercase();

        if !fname.ends_with(".csv") {
            continue;
        }

        let reader = io::BufReader::new(file);
        let mut first_line = true;
        for line in reader.lines() {
            let line = line.map_err(|e| DatjitError::Corpus(format!("read Census line: {e}")))?;
            if first_line {
                first_line = false;
                continue; // skip header
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let raw_name = parts[0].trim().trim_matches('"');
                // Skip aggregate categories
                if raw_name.contains(' ') {
                    continue;
                }
                // Title-case the name (Census data is ALL CAPS)
                let name = title_case(raw_name);
                // prop100k is the frequency per 100,000 people
                let weight: f64 = parts[3].trim().trim_matches('"').parse().unwrap_or(1.0);
                entries.push(CorpusEntry {
                    name,
                    weight,
                    gender: None,
                });
            }
        }
    }

    // Sort by weight descending, take top 5000
    entries.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    entries.truncate(5000);

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize Census: {e}")))?;
    let path = dest_dir.join("person_last.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write Census: {e}")))?;

    Ok(json.len() as u64)
}

/// Download GeoNames cities15000.zip and produce cities.json.
/// Tab-separated format: geonameid, name, asciiname, ..., country, ..., population, ..., timezone
fn download_and_process_geonames(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://download.geonames.org/export/dump/cities15000.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip GeoNames: {e}")))?;

    let mut cities: Vec<CityEntry> = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_string();

        if !fname.ends_with(".txt") {
            continue;
        }

        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| DatjitError::Corpus(format!("read GeoNames line: {e}")))?;
            let fields: Vec<&str> = line.split('\t').collect();
            // GeoNames format: 0=geonameid, 1=name, 2=asciiname, ..., 4=lat, 5=lng,
            //   8=country, 10=admin1, 14=population, 17=timezone
            if fields.len() < 18 {
                continue;
            }
            let population: u64 = fields[14].parse().unwrap_or(0);
            if population < 15000 {
                continue;
            }
            cities.push(CityEntry {
                name: fields[1].to_string(),
                ascii_name: fields[2].to_string(),
                lat: fields[4].parse().unwrap_or(0.0),
                lng: fields[5].parse().unwrap_or(0.0),
                country: fields[8].to_string(),
                admin1: fields[10].to_string(),
                population,
                timezone: fields[17].to_string(),
            });
        }
    }

    // Sort by population descending
    cities.sort_by(|a, b| b.population.cmp(&a.population));

    let json = serde_json::to_string_pretty(&cities)
        .map_err(|e| DatjitError::Corpus(format!("serialize GeoNames: {e}")))?;
    let path = dest_dir.join("cities.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write GeoNames: {e}")))?;

    Ok(json.len() as u64)
}

/// Download O*NET occupation data and alternate titles, produce job_titles.json.
fn download_and_process_onet(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Download main occupation data (tab-separated: SOC Code, Title, Description)
    let occ_data = download(
        client,
        "https://www.onetcenter.org/dl_files/database/db_28_3_text/Occupation%20Data.txt",
    )?;
    let occ_text = String::from_utf8_lossy(&occ_data);

    let mut entries: Vec<JobEntry> = Vec::new();
    let mut first = true;
    for line in occ_text.lines() {
        if first {
            first = false;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 3 {
            entries.push(JobEntry {
                soc_code: fields[0].to_string(),
                title: fields[1].to_string(),
                description: fields[2].chars().take(200).collect(),
                zone: None,
            });
        }
    }

    // Download job zones and merge
    let zone_data = download(
        client,
        "https://www.onetcenter.org/dl_files/database/db_28_3_text/Job%20Zones.txt",
    )?;
    let zone_text = String::from_utf8_lossy(&zone_data);
    let mut zone_map: HashMap<String, u8> = HashMap::new();
    let mut first_zone = true;
    for line in zone_text.lines() {
        if first_zone {
            first_zone = false;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 2 {
            if let Ok(z) = fields[1].trim().parse::<u8>() {
                zone_map.insert(fields[0].to_string(), z);
            }
        }
    }
    for entry in &mut entries {
        entry.zone = zone_map.get(&entry.soc_code).copied();
    }

    // Download alternate titles
    let alt_data = download(
        client,
        "https://www.onetcenter.org/dl_files/database/db_28_3_text/Alternate%20Titles.txt",
    )?;
    let alt_text = String::from_utf8_lossy(&alt_data);

    let mut alt_titles: Vec<AltTitleEntry> = Vec::new();
    let mut first = true;
    for line in alt_text.lines() {
        if first {
            first = false;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 2 {
            alt_titles.push(AltTitleEntry {
                soc_code: fields[0].to_string(),
                title: fields[1].to_string(),
            });
        }
    }

    let combined = JobCorpus {
        occupations: entries,
        alternate_titles: alt_titles,
    };

    let json = serde_json::to_string_pretty(&combined)
        .map_err(|e| DatjitError::Corpus(format!("serialize O*NET: {e}")))?;
    let path = dest_dir.join("job_titles.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write O*NET: {e}")))?;

    Ok(json.len() as u64)
}

/// Download IANA timezone database and extract timezone list.
fn download_and_process_timezones(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Instead of parsing the tarball, download the zone list from a simpler source
    // The GeoNames timeZones.txt is simpler and publicly accessible
    let data = download(
        client,
        "https://download.geonames.org/export/dump/timeZones.txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut timezones: Vec<TimezoneEntry> = Vec::new();
    let mut first = true;
    for line in text.lines() {
        if first {
            first = false;
            continue; // skip header
        }
        let fields: Vec<&str> = line.split('\t').collect();
        // Format: CountryCode, TimeZoneId, GMT offset, DST offset, raw offset
        if fields.len() >= 3 {
            let tz_id = fields[1].to_string();
            let country = fields[0].to_string();
            let gmt_offset: f64 = fields[2].parse().unwrap_or(0.0);
            if !tz_id.is_empty() && tz_id.contains('/') {
                timezones.push(TimezoneEntry {
                    timezone: tz_id,
                    country,
                    gmt_offset,
                });
            }
        }
    }

    // Deduplicate by timezone name
    timezones.sort_by(|a, b| a.timezone.cmp(&b.timezone));
    timezones.dedup_by(|a, b| a.timezone == b.timezone);

    let json = serde_json::to_string_pretty(&timezones)
        .map_err(|e| DatjitError::Corpus(format!("serialize timezones: {e}")))?;
    let path = dest_dir.join("timezones.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write timezones: {e}")))?;

    Ok(json.len() as u64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityEntry {
    pub name: String,
    pub ascii_name: String,
    pub lat: f64,
    pub lng: f64,
    pub country: String,
    pub admin1: String,
    pub population: u64,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEntry {
    pub soc_code: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltTitleEntry {
    pub soc_code: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCorpus {
    pub occupations: Vec<JobEntry>,
    pub alternate_titles: Vec<AltTitleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneEntry {
    pub timezone: String,
    pub country: String,
    pub gmt_offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryEntry {
    pub code: String,
    pub iso3: String,
    pub name: String,
    pub capital: String,
    pub population: u64,
    pub continent: String,
    pub currency_code: String,
    pub currency_name: String,
    pub phone_prefix: String,
    pub languages: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin1Entry {
    pub country: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostalCodeEntry {
    pub zip: String,
    pub city: String,
    pub state: String,
    pub state_code: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneFormatEntry {
    pub country_id: String,
    pub country_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_fixed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_mobile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardBinEntry {
    pub bin_prefix: String,
    pub brand: String,
    pub card_type: String,
    pub country: String,
}

/// Download GeoNames country info and produce countries.json.
fn download_and_process_countries(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://download.geonames.org/export/dump/countryInfo.txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut countries: Vec<CountryEntry> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 16 {
            continue;
        }
        let population: u64 = fields[7].parse().unwrap_or(0);
        countries.push(CountryEntry {
            code: fields[0].to_string(),
            iso3: fields[1].to_string(),
            name: fields[4].to_string(),
            capital: fields[5].to_string(),
            population,
            continent: fields[8].to_string(),
            currency_code: fields[10].to_string(),
            currency_name: fields[11].to_string(),
            phone_prefix: fields[12].to_string(),
            languages: fields[15].to_string(),
        });
    }

    countries.sort_by(|a, b| b.population.cmp(&a.population));

    let json = serde_json::to_string_pretty(&countries)
        .map_err(|e| DatjitError::Corpus(format!("serialize countries: {e}")))?;
    let path = dest_dir.join("countries.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write countries: {e}")))?;

    Ok(json.len() as u64)
}

/// Download GeoNames admin1 codes and produce admin1.json.
fn download_and_process_admin1(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://download.geonames.org/export/dump/admin1CodesASCII.txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut regions: Vec<Admin1Entry> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 2 {
            continue;
        }
        // Code is like "US.CA"
        let code_parts: Vec<&str> = fields[0].split('.').collect();
        if code_parts.len() < 2 {
            continue;
        }
        regions.push(Admin1Entry {
            country: code_parts[0].to_string(),
            code: code_parts[1].to_string(),
            name: fields[1].to_string(),
        });
    }

    regions.sort_by(|a, b| a.country.cmp(&b.country).then(a.code.cmp(&b.code)));

    let json = serde_json::to_string_pretty(&regions)
        .map_err(|e| DatjitError::Corpus(format!("serialize admin1: {e}")))?;
    let path = dest_dir.join("admin1.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write admin1: {e}")))?;

    Ok(json.len() as u64)
}

/// Download US postal code data from GeoNames and produce postal_codes.json.
fn download_and_process_postal_codes(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(client, "https://download.geonames.org/export/zip/US.zip")?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip US postal codes: {e}")))?;

    let mut entries: Vec<PostalCodeEntry> = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_string();

        if !fname.ends_with(".txt") {
            continue;
        }

        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line =
                line.map_err(|e| DatjitError::Corpus(format!("read postal code line: {e}")))?;
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 11 {
                continue;
            }
            let lat: f64 = fields[9].parse().unwrap_or(0.0);
            let lng: f64 = fields[10].parse().unwrap_or(0.0);
            entries.push(PostalCodeEntry {
                zip: fields[1].to_string(),
                city: fields[2].to_string(),
                state: fields[3].to_string(),
                state_code: fields[4].to_string(),
                lat,
                lng,
            });
        }
    }

    entries.sort_by(|a, b| a.zip.cmp(&b.zip));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize postal codes: {e}")))?;
    let path = dest_dir.join("postal_codes.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write postal codes: {e}")))?;

    Ok(json.len() as u64)
}

/// Download Google product taxonomy and produce product_categories.json.
fn download_and_process_product_categories(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www.google.com/basepages/producttype/taxonomy-with-ids.en-US.txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut categories: Vec<ProductCategory> = Vec::new();
    let mut first = true;
    for line in text.lines() {
        if first {
            first = false;
            continue; // skip header
        }
        if line.trim().is_empty() {
            continue;
        }
        // Format: "1 - Animals & Pet Supplies"
        if let Some(sep_pos) = line.find(" - ") {
            let id_str = line[..sep_pos].trim();
            let path = line[sep_pos + 3..].trim().to_string();
            if let Ok(id) = id_str.parse::<u32>() {
                let name = path.rsplit(" > ").next().unwrap_or(&path).to_string();
                categories.push(ProductCategory { id, name, path });
            }
        }
    }

    let json = serde_json::to_string_pretty(&categories)
        .map_err(|e| DatjitError::Corpus(format!("serialize product categories: {e}")))?;
    let path = dest_dir.join("product_categories.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write product categories: {e}")))?;

    Ok(json.len() as u64)
}

/// Download libphonenumber metadata XML and produce phone_formats.json.
fn download_and_process_phone_formats(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/google/libphonenumber/master/resources/PhoneNumberMetadata.xml",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<PhoneFormatEntry> = Vec::new();
    let mut reader = Reader::from_str(&text);

    // State tracking
    let mut current_country_id = String::new();
    let mut current_country_code = String::new();
    let mut current_example_fixed: Option<String> = None;
    let mut current_example_mobile: Option<String> = None;
    let mut in_territory = false;
    let mut in_fixed_line = false;
    let mut in_mobile = false;
    let mut in_example_number = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"territory" => {
                        in_territory = true;
                        current_country_id = String::new();
                        current_country_code = String::new();
                        current_example_fixed = None;
                        current_example_mobile = None;
                        in_fixed_line = false;
                        in_mobile = false;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    current_country_id =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"countryCode" => {
                                    current_country_code =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                                _ => {}
                            }
                        }
                    }
                    b"fixedLine" if in_territory => {
                        in_fixed_line = true;
                    }
                    b"mobile" if in_territory => {
                        in_mobile = true;
                    }
                    b"exampleNumber" if in_territory => {
                        in_example_number = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_example_number && in_territory {
                    let text_val = e.unescape().unwrap_or_default().trim().to_string();
                    if !text_val.is_empty() {
                        if in_fixed_line {
                            current_example_fixed = Some(text_val);
                        } else if in_mobile {
                            current_example_mobile = Some(text_val);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"territory" => {
                        if !current_country_id.is_empty() && !current_country_code.is_empty() {
                            entries.push(PhoneFormatEntry {
                                country_id: current_country_id.clone(),
                                country_code: current_country_code.clone(),
                                example_fixed: current_example_fixed.take(),
                                example_mobile: current_example_mobile.take(),
                            });
                        }
                        in_territory = false;
                        in_fixed_line = false;
                        in_mobile = false;
                    }
                    b"fixedLine" => {
                        in_fixed_line = false;
                    }
                    b"mobile" => {
                        in_mobile = false;
                    }
                    b"exampleNumber" => {
                        in_example_number = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(DatjitError::Corpus(format!("parse phone XML: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    entries.sort_by(|a, b| a.country_id.cmp(&b.country_id));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize phone formats: {e}")))?;
    let path = dest_dir.join("phone_formats.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write phone formats: {e}")))?;

    Ok(json.len() as u64)
}

/// Download credit card BIN list and produce credit_card_bins.json.
fn download_and_process_credit_card_bins(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/iannuttall/binlist-data/master/binlist-data.csv",
    )?;
    let text = String::from_utf8_lossy(&data);

    let allowed_brands = ["VISA", "MASTERCARD", "AMERICAN EXPRESS", "DISCOVER"];
    let mut entries: Vec<CreditCardBinEntry> = Vec::new();
    let mut first = true;
    for line in text.lines() {
        if first {
            first = false;
            continue; // skip header
        }
        if line.trim().is_empty() {
            continue;
        }
        // CSV: bin,brand,type,category,issuer,isoCountry
        // Simple CSV split (fields should not contain commas in this dataset)
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 {
            continue;
        }
        let brand = fields[1].trim().to_uppercase();
        if !allowed_brands.contains(&brand.as_str()) {
            continue;
        }
        entries.push(CreditCardBinEntry {
            bin_prefix: fields[0].trim().to_string(),
            brand,
            card_type: fields[2].trim().to_string(),
            country: fields[5].trim().to_string(),
        });
    }

    // Truncate to top 1000 entries
    entries.truncate(1000);

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize credit card bins: {e}")))?;
    let path = dest_dir.join("credit_card_bins.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write credit card bins: {e}")))?;

    Ok(json.len() as u64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyEntry {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub decimal_digits: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub code: String,
    pub alpha3: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TldEntry {
    pub tld: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeTypeEntry {
    pub mime_type: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportEntry {
    pub iata_code: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub airport_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlineEntry {
    pub name: String,
    pub iata: String,
    pub icao: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleEntry {
    pub year: u16,
    pub make: String,
    pub model: String,
    pub fuel_type: String,
    pub vehicle_class: String,
    pub drive: String,
    pub cylinders: Option<u8>,
}

/// Download CLDR currency data (fractions + display names) and produce currencies.json.
fn download_and_process_currencies(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Download fractions data
    let fractions_data = download(
        client,
        "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/currencyData.json",
    )?;
    let fractions_json: serde_json::Value = serde_json::from_slice(&fractions_data)
        .map_err(|e| DatjitError::Corpus(format!("parse CLDR fractions JSON: {e}")))?;

    // Build a map of currency code -> decimal digits
    let mut digits_map: HashMap<String, u8> = HashMap::new();
    if let Some(fractions) = fractions_json
        .pointer("/supplemental/currencyData/fractions")
        .and_then(|v| v.as_object())
    {
        for (code, info) in fractions {
            if let Some(digits) = info.get("_digits").and_then(|d| d.as_str()) {
                if let Ok(d) = digits.parse::<u8>() {
                    digits_map.insert(code.clone(), d);
                }
            }
        }
    }

    // Download display names
    let names_data = download(
        client,
        "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-numbers-full/main/en/currencies.json",
    )?;
    let names_json: serde_json::Value = serde_json::from_slice(&names_data)
        .map_err(|e| DatjitError::Corpus(format!("parse CLDR currencies JSON: {e}")))?;

    let mut entries: Vec<CurrencyEntry> = Vec::new();
    if let Some(currencies) = names_json
        .pointer("/main/en/numbers/currencies")
        .and_then(|v| v.as_object())
    {
        for (code, info) in currencies {
            let name = info
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let symbol = info
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or(code)
                .to_string();
            let decimal_digits = digits_map.get(code).copied().unwrap_or(2);
            if !name.is_empty() {
                entries.push(CurrencyEntry {
                    code: code.clone(),
                    name,
                    symbol,
                    decimal_digits,
                });
            }
        }
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize currencies: {e}")))?;
    let path = dest_dir.join("currencies.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write currencies: {e}")))?;

    Ok(json.len() as u64)
}

/// Download ISO 639 language codes and produce languages.json.
fn download_and_process_languages(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www.loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt",
    )?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<LanguageEntry> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Fields: alpha3_bib|alpha3_term|alpha2|english_name|french_name
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() < 5 {
            continue;
        }
        let alpha2 = fields[2].trim();
        if alpha2.is_empty() {
            continue;
        }
        let alpha3_bib = fields[0].trim();
        let english_name = fields[3].trim();
        entries.push(LanguageEntry {
            code: alpha2.to_string(),
            alpha3: alpha3_bib.to_string(),
            name: english_name.to_string(),
        });
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize languages: {e}")))?;
    let path = dest_dir.join("languages.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write languages: {e}")))?;

    Ok(json.len() as u64)
}

/// Download IANA TLD list and produce tlds.json.
fn download_and_process_tlds(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(client, "https://data.iana.org/TLD/tlds-alpha-by-domain.txt")?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<TldEntry> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        entries.push(TldEntry {
            tld: format!(".{}", line.to_lowercase()),
        });
    }

    entries.sort_by(|a, b| a.tld.cmp(&b.tld));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize TLDs: {e}")))?;
    let path = dest_dir.join("tlds.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write TLDs: {e}")))?;

    Ok(json.len() as u64)
}

/// Download IANA media types XML and produce mime_types.json.
fn download_and_process_mime_types(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www.iana.org/assignments/media-types/media-types.xml",
    )?;
    let text = String::from_utf8_lossy(&data);

    let allowed_categories: HashSet<&str> = [
        "application",
        "audio",
        "font",
        "image",
        "message",
        "model",
        "multipart",
        "text",
        "video",
    ]
    .iter()
    .copied()
    .collect();

    let mut entries: Vec<MimeTypeEntry> = Vec::new();
    let mut reader = Reader::from_str(&text);

    let mut current_category = String::new();
    let mut in_record = false;
    let mut in_name = false;
    let mut current_name = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"registry" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_category = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    b"record" => {
                        in_record = true;
                        current_name.clear();
                    }
                    b"name" if in_record => {
                        in_name = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_name && in_record {
                    let text_val = e.unescape().unwrap_or_default().trim().to_string();
                    current_name.push_str(&text_val);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"record" => {
                        if in_record
                            && !current_name.is_empty()
                            && allowed_categories.contains(current_category.as_str())
                        {
                            entries.push(MimeTypeEntry {
                                mime_type: format!("{}/{}", current_category, current_name),
                                category: current_category.clone(),
                            });
                        }
                        in_record = false;
                        in_name = false;
                        current_name.clear();
                    }
                    b"name" => {
                        in_name = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(DatjitError::Corpus(format!("parse MIME types XML: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    entries.sort_by(|a, b| a.mime_type.cmp(&b.mime_type));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize MIME types: {e}")))?;
    let path = dest_dir.join("mime_types.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write MIME types: {e}")))?;

    Ok(json.len() as u64)
}

/// Download OurAirports data and produce airports.json.
fn download_and_process_airports(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://davidmegginson.github.io/ourairports-data/airports.csv",
    )?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(io::Cursor::new(&data));

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read airports CSV headers: {e}")))?
        .clone();

    let col = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name) };
    let c_iata =
        col("iata_code").ok_or_else(|| DatjitError::Corpus("missing iata_code column".into()))?;
    let c_name = col("name").ok_or_else(|| DatjitError::Corpus("missing name column".into()))?;
    let c_city = col("municipality")
        .ok_or_else(|| DatjitError::Corpus("missing municipality column".into()))?;
    let c_country = col("iso_country")
        .ok_or_else(|| DatjitError::Corpus("missing iso_country column".into()))?;
    let c_lat = col("latitude_deg")
        .ok_or_else(|| DatjitError::Corpus("missing latitude_deg column".into()))?;
    let c_lng = col("longitude_deg")
        .ok_or_else(|| DatjitError::Corpus("missing longitude_deg column".into()))?;
    let c_type = col("type").ok_or_else(|| DatjitError::Corpus("missing type column".into()))?;

    let allowed_types: HashSet<&str> = ["large_airport", "medium_airport"]
        .iter()
        .copied()
        .collect();

    let mut entries: Vec<AirportEntry> = Vec::new();
    for result in rdr.records() {
        let record =
            result.map_err(|e| DatjitError::Corpus(format!("read airports CSV record: {e}")))?;
        let iata = record.get(c_iata).unwrap_or("").trim();
        let atype = record.get(c_type).unwrap_or("").trim();
        if iata.is_empty() || !allowed_types.contains(atype) {
            continue;
        }
        entries.push(AirportEntry {
            iata_code: iata.to_string(),
            name: record.get(c_name).unwrap_or("").trim().to_string(),
            city: record.get(c_city).unwrap_or("").trim().to_string(),
            country: record.get(c_country).unwrap_or("").trim().to_string(),
            lat: record
                .get(c_lat)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0.0),
            lng: record
                .get(c_lng)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0.0),
            airport_type: atype.to_string(),
        });
    }

    entries.sort_by(|a, b| a.iata_code.cmp(&b.iata_code));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize airports: {e}")))?;
    let path = dest_dir.join("airports.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write airports: {e}")))?;

    Ok(json.len() as u64)
}

/// Download OpenFlights airline data and produce airlines.json.
fn download_and_process_airlines(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://raw.githubusercontent.com/jpatokal/openflights/master/data/airlines.dat",
    )?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(io::Cursor::new(&data));

    let mut entries: Vec<AirlineEntry> = Vec::new();
    for result in rdr.records() {
        let record =
            result.map_err(|e| DatjitError::Corpus(format!("read airlines CSV record: {e}")))?;
        // Fields: 0=id, 1=name, 2=alias, 3=iata, 4=icao, 5=callsign, 6=country, 7=active
        if record.len() < 8 {
            continue;
        }
        let active = record.get(7).unwrap_or("").trim();
        let iata = record.get(3).unwrap_or("").trim();
        if active != "Y" || iata.is_empty() || iata == "\\N" || iata == "-" {
            continue;
        }
        entries.push(AirlineEntry {
            name: record.get(1).unwrap_or("").trim().to_string(),
            iata: iata.to_string(),
            icao: record.get(4).unwrap_or("").trim().to_string(),
            country: record.get(6).unwrap_or("").trim().to_string(),
        });
    }

    entries.sort_by(|a, b| a.iata.cmp(&b.iata));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize airlines: {e}")))?;
    let path = dest_dir.join("airlines.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write airlines: {e}")))?;

    Ok(json.len() as u64)
}

/// Download EPA fuel economy vehicle data and produce vehicles.json.
fn download_and_process_vehicles(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = download(
        client,
        "https://www.fueleconomy.gov/feg/epadata/vehicles.csv.zip",
    )?;
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| DatjitError::Corpus(format!("unzip EPA vehicles: {e}")))?;

    let mut csv_data: Vec<u8> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| DatjitError::Corpus(format!("zip entry: {e}")))?;
        let fname = file.name().to_lowercase();
        if fname.ends_with(".csv") {
            io::Read::read_to_end(&mut file, &mut csv_data)
                .map_err(|e| DatjitError::Corpus(format!("read vehicles CSV from zip: {e}")))?;
            break;
        }
    }

    if csv_data.is_empty() {
        return Err(DatjitError::Corpus("no CSV found in vehicles zip".into()));
    }

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(io::Cursor::new(&csv_data));

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read vehicles CSV headers: {e}")))?
        .clone();

    let col = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name) };
    let c_year = col("year").ok_or_else(|| DatjitError::Corpus("missing year column".into()))?;
    let c_make = col("make").ok_or_else(|| DatjitError::Corpus("missing make column".into()))?;
    let c_model = col("model").ok_or_else(|| DatjitError::Corpus("missing model column".into()))?;
    let c_fuel = col("fuelType1").or_else(|| col("fuelType"));
    let c_class = col("VClass");
    let c_drive = col("drive");
    let c_cyl = col("cylinders");

    let mut entries: Vec<VehicleEntry> = Vec::new();
    let mut seen: HashSet<(String, String, u16)> = HashSet::new();

    for result in rdr.records() {
        let record =
            result.map_err(|e| DatjitError::Corpus(format!("read vehicles CSV record: {e}")))?;
        let year: u16 = record
            .get(c_year)
            .unwrap_or("0")
            .trim()
            .parse()
            .unwrap_or(0);
        if year < 2015 {
            continue;
        }
        let make = record.get(c_make).unwrap_or("").trim().to_string();
        let model = record.get(c_model).unwrap_or("").trim().to_string();
        let key = (make.clone(), model.clone(), year);
        if !seen.insert(key) {
            continue;
        }
        let fuel_type = c_fuel
            .and_then(|c| record.get(c))
            .unwrap_or("")
            .trim()
            .to_string();
        let vehicle_class = c_class
            .and_then(|c| record.get(c))
            .unwrap_or("")
            .trim()
            .to_string();
        let drive = c_drive
            .and_then(|c| record.get(c))
            .unwrap_or("")
            .trim()
            .to_string();
        let cylinders: Option<u8> = c_cyl
            .and_then(|c| record.get(c))
            .and_then(|v| v.trim().parse().ok());

        entries.push(VehicleEntry {
            year,
            make,
            model,
            fuel_type,
            vehicle_class,
            drive,
            cylinders,
        });

        if entries.len() >= 5000 {
            break;
        }
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize vehicles: {e}")))?;
    let path = dest_dir.join("vehicles.json");
    fs::write(&path, &json).map_err(|e| DatjitError::Corpus(format!("write vehicles: {e}")))?;

    Ok(json.len() as u64)
}

fn title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_sources_count() {
        let sources = known_sources();
        assert_eq!(sources.len(), 38);
    }

    #[test]
    fn test_known_sources_categories() {
        let sources = known_sources();
        let categories: Vec<&str> = sources.iter().map(|s| s.category.as_str()).collect();
        assert!(categories.contains(&"person"));
        assert!(categories.contains(&"address"));
        assert!(categories.contains(&"job"));
        assert!(categories.contains(&"shared"));
        assert!(categories.contains(&"product"));
        assert!(categories.contains(&"company"));
    }

    #[test]
    fn test_default_corpus_dir() {
        let dir = default_corpus_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.ends_with(".datjit/corpus") || dir_str.ends_with(".datjit\\corpus"));
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("SMITH"), "Smith");
        assert_eq!(title_case("GARCIA"), "Garcia");
        assert_eq!(title_case("o'brien"), "O'brien");
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn test_check_corpus_status_nonexistent() {
        std::env::set_var("HOME", "/tmp/datjit_test_nonexistent_home_12345");
        let status = check_corpus_status().unwrap();
        assert!(status.installed_locales.is_empty());
        assert!(status.installed_files.is_empty());
        assert_eq!(status.total_size_bytes, 0);
    }
}
