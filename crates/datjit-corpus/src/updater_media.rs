//! Batch 9: Media & entertainment datasets.
//!
//! Downloads media catalog data from public domain / open license sources:
//! - MusicBrainz (CC0): artist names from PostgreSQL dump
//! - Discogs (CC0): artists and labels from monthly XML dumps
//! - Open Library (CC0): book editions from bulk TSV dump
//! - IMDb (non-commercial): titles and names from daily TSV dumps

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use datjit_core::error::DatjitError;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::updater::{download_source, CorpusSource, CorpusUpdateReport};

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicBrainzArtistEntry {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub sort_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub artist_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsArtistEntry {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscogsLabelEntry {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLibraryBookEntry {
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub publisher: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub publish_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbTitleEntry {
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub genres: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_minutes: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbNameEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<u16>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub professions: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

pub fn media_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "MusicBrainz Artists".into(),
            description: "Artist names from MusicBrainz open music database".into(),
            url: "https://data.metabrainz.org/pub/musicbrainz/data/fullexport/".into(),
            license: "CC0".into(),
            category: "media".into(),
        },
        CorpusSource {
            name: "Discogs Artists".into(),
            description: "Artist names from Discogs monthly data dumps".into(),
            url: "https://discogs-data-dumps.s3-us-west-2.amazonaws.com/data/".into(),
            license: "CC0".into(),
            category: "media".into(),
        },
        CorpusSource {
            name: "Discogs Labels".into(),
            description: "Record label names from Discogs monthly data dumps".into(),
            url: "https://discogs-data-dumps.s3-us-west-2.amazonaws.com/data/".into(),
            license: "CC0".into(),
            category: "media".into(),
        },
        CorpusSource {
            name: "Open Library Books".into(),
            description: "Book editions from Open Library bulk data dumps".into(),
            url: "https://openlibrary.org/data/ol_dump_editions_latest.txt.gz".into(),
            license: "CC0".into(),
            category: "media".into(),
        },
        CorpusSource {
            name: "IMDb Titles".into(),
            description: "Movie and TV titles from IMDb non-commercial datasets".into(),
            url: "https://datasets.imdbws.com/title.basics.tsv.gz".into(),
            license: "Non-commercial".into(),
            category: "media".into(),
        },
        CorpusSource {
            name: "IMDb Names".into(),
            description: "Actor and crew names from IMDb non-commercial datasets".into(),
            url: "https://datasets.imdbws.com/name.basics.tsv.gz".into(),
            license: "Non-commercial".into(),
            category: "media".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub fn download_media_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "MusicBrainz Artists",
        "shared/musicbrainz_artists.json",
        || download_and_process_musicbrainz_artists(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Discogs Artists",
        "shared/discogs_artists.json",
        || download_and_process_discogs(client, temp_shared, "artists"),
        report,
        on_progress,
    );

    download_source(
        "Discogs Labels",
        "shared/discogs_labels.json",
        || download_and_process_discogs(client, temp_shared, "labels"),
        report,
        on_progress,
    );

    download_source(
        "Open Library Books",
        "shared/openlibrary_books.json",
        || download_and_process_openlibrary(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "IMDb Titles",
        "shared/imdb_titles.json",
        || download_and_process_imdb_titles(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "IMDb Names",
        "shared/imdb_names.json",
        || download_and_process_imdb_names(client, temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// MusicBrainz Artists (streaming bzip2 tar archive)
// ---------------------------------------------------------------------------

fn download_and_process_musicbrainz_artists(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    // Fetch the LATEST date string to construct the dump URL
    let latest_url = "https://data.metabrainz.org/pub/musicbrainz/data/fullexport/LATEST";
    let resp = client
        .get(latest_url)
        .send()
        .map_err(|e| DatjitError::Corpus(format!("fetch MusicBrainz LATEST: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for MusicBrainz LATEST",
            resp.status()
        )));
    }

    let date = resp
        .text()
        .map_err(|e| DatjitError::Corpus(format!("read MusicBrainz LATEST: {e}")))?
        .trim()
        .to_string();

    let dump_url = format!(
        "https://data.metabrainz.org/pub/musicbrainz/data/fullexport/{date}/mbdump.tar.bz2"
    );

    let resp = client
        .get(&dump_url)
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download MusicBrainz dump: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for MusicBrainz dump",
            resp.status()
        )));
    }

    // Stream: HTTP response -> bzip2 decoder -> tar archive
    let bz_decoder = bzip2::read::BzDecoder::new(resp);
    let mut archive = tar::Archive::new(bz_decoder);

    let mut entries: Vec<MusicBrainzArtistEntry> = Vec::new();
    let max_entries = 20_000;

    // Artist type mapping (from MusicBrainz schema)
    let type_name = |id: &str| -> &str {
        match id {
            "1" => "Person",
            "2" => "Group",
            "3" => "Orchestra",
            "4" => "Choir",
            "5" => "Character",
            "6" => "Other",
            _ => "",
        }
    };

    for entry_result in archive.entries().map_err(|e| {
        DatjitError::Corpus(format!("read MusicBrainz tar entries: {e}"))
    })? {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if !path.ends_with("/artist") {
            continue;
        }

        // Found the artist file — parse PostgreSQL COPY format (tab-separated)
        // Columns: id, gid, name, sort_name, begin_date_year, begin_date_month,
        //          begin_date_day, end_date_year, end_date_month, end_date_day,
        //          type, area, gender, comment, edits_pending, last_updated, ended
        let reader = BufReader::new(entry);
        for line in reader.lines() {
            if entries.len() >= max_entries {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 4 {
                continue;
            }

            let name = cols[2].trim().to_string();
            if name.is_empty() || name.len() < 2 {
                continue;
            }
            // Skip entries that look like database control lines
            if name.starts_with('\\') || name == "\\." {
                continue;
            }
            // Skip non-Latin names for corpus consistency
            if !name.chars().any(|c| c.is_ascii_alphabetic()) {
                continue;
            }

            let sort_name = cols[3].trim().to_string();
            let artist_type = cols
                .get(10)
                .map(|t| type_name(t.trim()).to_string())
                .unwrap_or_default();

            entries.push(MusicBrainzArtistEntry {
                name,
                sort_name,
                artist_type,
            });
        }
        break; // Only process the artist file
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("musicbrainz_artists.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write musicbrainz_artists.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// Discogs Artists & Labels (streaming XML.gz)
// ---------------------------------------------------------------------------

fn download_and_process_discogs(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
    data_type: &str, // "artists" or "labels"
) -> Result<u64, DatjitError> {
    // Discogs publishes monthly dumps. Try recent dates to find a valid URL.
    let resp = try_discogs_download(client, data_type)?;

    let decoder = flate2::read::GzDecoder::new(resp);
    let mut reader = Reader::from_reader(BufReader::new(decoder));
    reader.config_mut().trim_text(true);

    let max_entries = if data_type == "labels" {
        10_000
    } else {
        20_000
    };

    let mut buf = Vec::new();
    let mut in_element = false;
    let mut current_name = String::new();
    let mut in_name = false;

    // The tag we're looking for: <artist> or <label>
    let element_tag = if data_type == "artists" {
        b"artist".as_slice()
    } else {
        b"label".as_slice()
    };

    if data_type == "artists" {
        let mut entries: Vec<DiscogsArtistEntry> = Vec::new();
        loop {
            if entries.len() >= max_entries {
                break;
            }
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == element_tag {
                        in_element = true;
                        current_name.clear();
                    } else if in_element && local.as_ref() == b"name" {
                        in_name = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_name {
                        if let Ok(text) = e.unescape() {
                            current_name = text.trim().to_string();
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"name" {
                        in_name = false;
                    } else if local.as_ref() == element_tag {
                        if !current_name.is_empty()
                            && current_name.len() >= 2
                            && current_name.chars().any(|c| c.is_ascii_alphabetic())
                        {
                            entries.push(DiscogsArtistEntry {
                                name: current_name.clone(),
                            });
                        }
                        in_element = false;
                        current_name.clear();
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
        let path = dest_dir.join("discogs_artists.json");
        fs::write(&path, &json)
            .map_err(|e| DatjitError::Corpus(format!("write discogs_artists.json: {e}")))?;
        Ok(json.len() as u64)
    } else {
        let mut entries: Vec<DiscogsLabelEntry> = Vec::new();
        loop {
            if entries.len() >= max_entries {
                break;
            }
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == element_tag {
                        in_element = true;
                        current_name.clear();
                    } else if in_element && local.as_ref() == b"name" {
                        in_name = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_name {
                        if let Ok(text) = e.unescape() {
                            current_name = text.trim().to_string();
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"name" {
                        in_name = false;
                    } else if local.as_ref() == element_tag {
                        if !current_name.is_empty()
                            && current_name.len() >= 2
                            && current_name.chars().any(|c| c.is_ascii_alphabetic())
                        {
                            entries.push(DiscogsLabelEntry {
                                name: current_name.clone(),
                            });
                        }
                        in_element = false;
                        current_name.clear();
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
        let path = dest_dir.join("discogs_labels.json");
        fs::write(&path, &json)
            .map_err(|e| DatjitError::Corpus(format!("write discogs_labels.json: {e}")))?;
        Ok(json.len() as u64)
    }
}

/// Try to find and download a recent Discogs monthly dump.
/// Discogs publishes on the 1st of each month.
fn try_discogs_download(
    client: &reqwest::blocking::Client,
    data_type: &str,
) -> Result<reqwest::blocking::Response, DatjitError> {
    // Try dates from recent months going backwards
    let candidates = [
        "2026/discogs_20260401",
        "2026/discogs_20260301",
        "2026/discogs_20260201",
        "2026/discogs_20260101",
        "2025/discogs_20251201",
        "2025/discogs_20251101",
        "2025/discogs_20251001",
        "2025/discogs_20250901",
        "2025/discogs_20250801",
        "2025/discogs_20250701",
        "2025/discogs_20250601",
        "2025/discogs_20250501",
        "2025/discogs_20250401",
        "2025/discogs_20250301",
        "2025/discogs_20250201",
        "2025/discogs_20250101",
    ];

    for candidate in &candidates {
        let url = format!(
            "https://discogs-data-dumps.s3-us-west-2.amazonaws.com/data/{candidate}_{data_type}.xml.gz"
        );

        match client.head(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                // Found a valid URL — now do the actual GET
                let resp = client.get(&url).send().map_err(|e| {
                    DatjitError::Corpus(format!("download Discogs {data_type}: {e}"))
                })?;
                if resp.status().is_success() {
                    return Ok(resp);
                }
            }
            _ => continue,
        }
    }

    Err(DatjitError::Corpus(format!(
        "could not find recent Discogs {data_type} dump (tried {} dates)",
        candidates.len()
    )))
}

// ---------------------------------------------------------------------------
// Open Library (streaming TSV.gz with embedded JSON)
// ---------------------------------------------------------------------------

fn download_and_process_openlibrary(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://openlibrary.org/data/ol_dump_editions_latest.txt.gz")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download Open Library: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for Open Library",
            resp.status()
        )));
    }

    // Format: type\tkey\trevision\tlast_modified\tjson
    let decoder = flate2::read::GzDecoder::new(resp);
    let reader = BufReader::new(decoder);

    let mut entries: Vec<OpenLibraryBookEntry> = Vec::new();
    let max_entries = 30_000;

    for line in reader.lines() {
        if entries.len() >= max_entries {
            break;
        }
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        // Split into 5 tab-separated columns; JSON is in column 5 (index 4)
        let cols: Vec<&str> = line.splitn(5, '\t').collect();
        if cols.len() < 5 {
            continue;
        }

        let json_str = cols[4];
        let val: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let title = val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if title.is_empty() || title.len() < 3 {
            continue;
        }
        if !title.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        // Extract first publisher from publishers array
        let publisher = val
            .get("publishers")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|p| {
                // Can be a string or {"name": "..."} object
                p.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            })
            .unwrap_or_default();

        let publish_date = val
            .get("publish_date")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        entries.push(OpenLibraryBookEntry {
            title,
            publisher,
            publish_date,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("openlibrary_books.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write openlibrary_books.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// IMDb Titles (streaming TSV.gz)
// ---------------------------------------------------------------------------

fn download_and_process_imdb_titles(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://datasets.imdbws.com/title.basics.tsv.gz")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download IMDb titles: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for IMDb titles",
            resp.status()
        )));
    }

    // Columns: tconst, titleType, primaryTitle, originalTitle, isAdult, startYear,
    //          endYear, runtimeMinutes, genres
    let decoder = flate2::read::GzDecoder::new(resp);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b'\t')
        .from_reader(decoder);

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("IMDb titles headers: {e}")))?
        .clone();

    let title_idx = headers.iter().position(|h| h == "primaryTitle");
    let type_idx = headers.iter().position(|h| h == "titleType");
    let adult_idx = headers.iter().position(|h| h == "isAdult");
    let year_idx = headers.iter().position(|h| h == "startYear");
    let runtime_idx = headers.iter().position(|h| h == "runtimeMinutes");
    let genres_idx = headers.iter().position(|h| h == "genres");

    let title_idx = title_idx
        .ok_or_else(|| DatjitError::Corpus("IMDb: missing primaryTitle column".into()))?;

    let mut entries: Vec<ImdbTitleEntry> = Vec::new();
    let max_entries = 30_000;

    for result in rdr.records() {
        if entries.len() >= max_entries {
            break;
        }
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Skip adult content
        let is_adult = adult_idx
            .and_then(|i| record.get(i))
            .unwrap_or("0");
        if is_adult == "1" {
            continue;
        }

        let title = record.get(title_idx).unwrap_or("").trim().to_string();
        if title.is_empty() || title.len() < 2 {
            continue;
        }

        let title_type = type_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();

        let year = year_idx
            .and_then(|i| record.get(i))
            .and_then(|s| s.parse::<u16>().ok());

        let runtime_minutes = runtime_idx
            .and_then(|i| record.get(i))
            .and_then(|s| s.parse::<u16>().ok());

        let genres = genres_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        // IMDb uses \N for null values
        let genres = if genres == "\\N" {
            String::new()
        } else {
            genres
        };

        entries.push(ImdbTitleEntry {
            title,
            title_type,
            year,
            genres,
            runtime_minutes,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("imdb_titles.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write imdb_titles.json: {e}")))?;

    Ok(json.len() as u64)
}

// ---------------------------------------------------------------------------
// IMDb Names (streaming TSV.gz)
// ---------------------------------------------------------------------------

fn download_and_process_imdb_names(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let resp = client
        .get("https://datasets.imdbws.com/name.basics.tsv.gz")
        .send()
        .map_err(|e| DatjitError::Corpus(format!("download IMDb names: {e}")))?;

    if !resp.status().is_success() {
        return Err(DatjitError::Corpus(format!(
            "HTTP {} for IMDb names",
            resp.status()
        )));
    }

    // Columns: nconst, primaryName, birthYear, deathYear, primaryProfession, knownForTitles
    let decoder = flate2::read::GzDecoder::new(resp);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b'\t')
        .from_reader(decoder);

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("IMDb names headers: {e}")))?
        .clone();

    let name_idx = headers.iter().position(|h| h == "primaryName");
    let birth_idx = headers.iter().position(|h| h == "birthYear");
    let prof_idx = headers.iter().position(|h| h == "primaryProfession");

    let name_idx =
        name_idx.ok_or_else(|| DatjitError::Corpus("IMDb: missing primaryName column".into()))?;

    let mut entries: Vec<ImdbNameEntry> = Vec::new();
    let max_entries = 20_000;

    for result in rdr.records() {
        if entries.len() >= max_entries {
            break;
        }
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        let name = record.get(name_idx).unwrap_or("").trim().to_string();
        if name.is_empty() || name.len() < 2 {
            continue;
        }

        let birth_year = birth_idx
            .and_then(|i| record.get(i))
            .and_then(|s| s.parse::<u16>().ok());

        let professions = prof_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        let professions = if professions == "\\N" {
            String::new()
        } else {
            professions
        };

        entries.push(ImdbNameEntry {
            name,
            birth_year,
            professions,
        });
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| DatjitError::Corpus(format!("JSON serialize: {e}")))?;
    let path = dest_dir.join("imdb_names.json");
    fs::write(&path, &json)
        .map_err(|e| DatjitError::Corpus(format!("write imdb_names.json: {e}")))?;

    Ok(json.len() as u64)
}
