use std::path::PathBuf;

use datjit_core::error::DatjitError;

/// Report of what was updated during a corpus update operation.
pub struct CorpusUpdateReport {
    pub locale: String,
    pub files_updated: Vec<String>,
    pub files_skipped: Vec<String>,
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

/// List all known corpus sources from the datjit-sources.md spec.
pub fn known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "SSA Baby Names".into(),
            description: "US Social Security Administration baby name frequency data".into(),
            url: "https://www.ssa.gov/oact/babynames/names.zip".into(),
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
            description: "O*NET occupational titles and alternate titles".into(),
            url: "https://www.onetcenter.org/dl_files/database/db_28_3_text/Occupation%20Data.txt"
                .into(),
            license: "CC BY 4.0".into(),
            category: "job".into(),
        },
        CorpusSource {
            name: "Faker.js Locales".into(),
            description: "Faker.js locale data for multi-locale synthetic data".into(),
            url: "https://github.com/faker-js/faker/tree/main/src/locales".into(),
            license: "MIT".into(),
            category: "multi".into(),
        },
        CorpusSource {
            name: "Google libphonenumber".into(),
            description: "Phone number metadata from Google libphonenumber".into(),
            url: "https://github.com/google/libphonenumber/tree/master/resources".into(),
            license: "Apache 2.0".into(),
            category: "phone".into(),
        },
        CorpusSource {
            name: "IANA Timezones".into(),
            description: "IANA time zone database".into(),
            url: "https://www.iana.org/time-zones".into(),
            license: "Public Domain".into(),
            category: "shared".into(),
        },
    ]
}

/// Get the default corpus directory (~/.datjit/corpus/).
pub fn default_corpus_dir() -> PathBuf {
    dirs_or_fallback()
}

fn dirs_or_fallback() -> PathBuf {
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

    // Scan the corpus directory for locale subdirectories and files
    let entries = std::fs::read_dir(&corpus_dir).map_err(DatjitError::Io)?;

    for entry in entries {
        let entry = entry.map_err(DatjitError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                installed_locales.push(name.to_string());
            }
            // Scan files inside locale dir
            if let Ok(sub_entries) = std::fs::read_dir(&path) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_sources_count() {
        let sources = known_sources();
        assert_eq!(sources.len(), 7);
    }

    #[test]
    fn test_known_sources_categories() {
        let sources = known_sources();
        let categories: Vec<&str> = sources.iter().map(|s| s.category.as_str()).collect();
        assert!(categories.contains(&"person"));
        assert!(categories.contains(&"address"));
        assert!(categories.contains(&"job"));
        assert!(categories.contains(&"phone"));
        assert!(categories.contains(&"shared"));
        assert!(categories.contains(&"multi"));
    }

    #[test]
    fn test_default_corpus_dir() {
        let dir = default_corpus_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.ends_with(".datjit/corpus") || dir_str.ends_with(".datjit\\corpus"));
    }

    #[test]
    fn test_check_corpus_status_nonexistent() {
        // When corpus dir doesn't exist, we should get an empty status
        std::env::set_var("HOME", "/tmp/datjit_test_nonexistent_home_12345");
        let status = check_corpus_status().unwrap();
        assert!(status.installed_locales.is_empty());
        assert!(status.installed_files.is_empty());
        assert_eq!(status.total_size_bytes, 0);
    }
}
