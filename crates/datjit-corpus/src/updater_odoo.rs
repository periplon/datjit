//! Batch 5: Odoo ERP default reference data.
//!
//! Downloads master data from the Odoo 17.0 GitHub repository (LGPL-3.0):
//! countries, states, currencies, units of measure, payment terms,
//! incoterms, tax rates, and account types.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use datjit_core::error::DatjitError;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::updater::{download, download_source, CorpusSource, CorpusUpdateReport};

const ODOO_RAW_BASE: &str = "https://raw.githubusercontent.com/odoo/odoo/17.0/";

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpCountryEntry {
    pub name: String,
    pub code: String,
    pub phone_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpStateEntry {
    pub name: String,
    pub code: String,
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpCurrencyEntry {
    pub name: String,
    pub code: String,
    pub symbol: String,
    pub rounding: f64,
    pub position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpUomEntry {
    pub name: String,
    pub category: String,
    pub uom_type: String,
    pub factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpPaymentTermEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpIncotermEntry {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpTaxRateEntry {
    pub name: String,
    pub rate: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_tax_use: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpAccountTypeEntry {
    pub name: String,
    pub account_type: String,
}

// ---------------------------------------------------------------------------
// Known sources
// ---------------------------------------------------------------------------

pub fn odoo_known_sources() -> Vec<CorpusSource> {
    vec![
        CorpusSource {
            name: "Odoo Countries".into(),
            description: "200+ countries with phone codes and VAT labels from Odoo ERP".into(),
            url: format!("{ODOO_RAW_BASE}odoo/addons/base/data/res_country_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo States/Provinces".into(),
            description: "700+ states/provinces for 40+ countries from Odoo ERP".into(),
            url: format!("{ODOO_RAW_BASE}odoo/addons/base/data/res.country.state.csv"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Currencies".into(),
            description: "190+ currencies with symbols and rounding from Odoo ERP".into(),
            url: format!("{ODOO_RAW_BASE}odoo/addons/base/data/res_currency_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Units of Measure".into(),
            description: "Units of measure with conversion factors from Odoo ERP".into(),
            url: format!("{ODOO_RAW_BASE}addons/uom/data/uom_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Payment Terms".into(),
            description: "Standard payment terms from Odoo ERP accounting".into(),
            url: format!("{ODOO_RAW_BASE}addons/account/data/account_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Incoterms".into(),
            description: "International commercial terms from Odoo ERP".into(),
            url: format!("{ODOO_RAW_BASE}addons/account/data/account_incoterms_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Tax Rates".into(),
            description: "Generic chart of accounts tax rates from Odoo ERP".into(),
            url: format!(
                "{ODOO_RAW_BASE}addons/l10n_generic_coa/data/account_tax_template_data.xml"
            ),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
        CorpusSource {
            name: "Odoo Account Types".into(),
            description: "Account type categories from Odoo ERP accounting".into(),
            url: format!("{ODOO_RAW_BASE}addons/account/data/account_data.xml"),
            license: "LGPL-3.0".into(),
            category: "erp".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub fn download_odoo_sources(
    client: &reqwest::blocking::Client,
    temp_shared: &Path,
    _temp_locale: &Path,
    report: &mut CorpusUpdateReport,
    on_progress: &dyn Fn(&str),
) {
    download_source(
        "Odoo Countries",
        "shared/erp_countries.json",
        || download_and_process_erp_countries(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo States/Provinces",
        "shared/erp_states.json",
        || download_and_process_erp_states(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo Currencies",
        "shared/erp_currencies.json",
        || download_and_process_erp_currencies(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo Units of Measure",
        "shared/erp_uom.json",
        || download_and_process_erp_uom(client, temp_shared),
        report,
        on_progress,
    );

    // Payment terms and account types share the same source file — download once
    let account_data_url = format!("{ODOO_RAW_BASE}addons/account/data/account_data.xml");
    let account_data = download(client, &account_data_url).ok();

    download_source(
        "Odoo Payment Terms",
        "shared/erp_payment_terms.json",
        || process_erp_payment_terms(account_data.as_deref(), temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo Account Types",
        "shared/erp_account_types.json",
        || process_erp_account_types(account_data.as_deref(), temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo Incoterms",
        "shared/erp_incoterms.json",
        || download_and_process_erp_incoterms(client, temp_shared),
        report,
        on_progress,
    );

    download_source(
        "Odoo Tax Rates",
        "shared/erp_tax_rates.json",
        || download_and_process_erp_tax_rates(client, temp_shared),
        report,
        on_progress,
    );
}

// ---------------------------------------------------------------------------
// Odoo XML parser
// ---------------------------------------------------------------------------

/// Parse Odoo XML and extract records for a given model.
/// Returns a list of field maps; foreign key refs are stored as `fieldname__ref`.
fn parse_odoo_xml_records(xml_text: &str, target_model: &str) -> Vec<HashMap<String, String>> {
    let mut reader = Reader::from_str(xml_text);
    reader.config_mut().trim_text(true);
    let mut records = Vec::new();
    let mut current_record: Option<HashMap<String, String>> = None;
    let mut current_field_name = String::new();
    let mut in_field = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"record" => {
                    let mut model = String::new();
                    let mut id = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"model" => model = String::from_utf8_lossy(&attr.value).to_string(),
                            b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                            _ => {}
                        }
                    }
                    if model == target_model {
                        let mut map = HashMap::new();
                        map.insert("__id__".into(), id);
                        current_record = Some(map);
                    }
                }
                b"field" if current_record.is_some() => {
                    current_field_name.clear();
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"name" => {
                                current_field_name =
                                    String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"ref" => {
                                if let Some(rec) = current_record.as_mut() {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    rec.insert(format!("{current_field_name}__ref"), val);
                                }
                            }
                            b"eval" => {
                                if let Some(rec) = current_record.as_mut() {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    rec.insert(current_field_name.clone(), val);
                                }
                            }
                            _ => {}
                        }
                    }
                    in_field = true;
                }
                _ => {}
            },
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"field" => {
                if current_record.is_some() {
                    let mut fname = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"name" => {
                                fname = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"ref" => {
                                if let Some(rec) = current_record.as_mut() {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    rec.insert(format!("{fname}__ref"), val);
                                }
                            }
                            b"eval" => {
                                if let Some(rec) = current_record.as_mut() {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    rec.insert(fname.clone(), val);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) if in_field && current_record.is_some() => {
                if let Ok(text) = e.unescape() {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        if let Some(rec) = current_record.as_mut() {
                            rec.insert(current_field_name.clone(), text);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"field" => {
                    in_field = false;
                }
                b"record" => {
                    if let Some(rec) = current_record.take() {
                        records.push(rec);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    records
}

/// Strip module prefix from an Odoo external ID: "base.EUR" → "EUR"
fn strip_module_prefix(ref_id: &str) -> &str {
    ref_id.rsplit('.').next().unwrap_or(ref_id)
}

fn write_json<T: Serialize>(
    entries: &[T],
    dest: &Path,
    filename: &str,
) -> Result<u64, DatjitError> {
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| DatjitError::Corpus(format!("serialize {filename}: {e}")))?;
    let size = json.len() as u64;
    fs::write(dest.join(filename), &json)
        .map_err(|e| DatjitError::Corpus(format!("write {filename}: {e}")))?;
    Ok(size)
}

// ---------------------------------------------------------------------------
// 1. Countries
// ---------------------------------------------------------------------------

fn download_and_process_erp_countries(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}odoo/addons/base/data/res_country_data.xml");
    let data = download(client, &url)?;
    let xml = String::from_utf8_lossy(&data);

    let records = parse_odoo_xml_records(&xml, "res.country");
    let entries: Vec<ErpCountryEntry> = records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?;
            let code = rec.get("code")?;
            Some(ErpCountryEntry {
                name: name.clone(),
                code: code.clone(),
                phone_code: rec.get("phone_code").cloned().unwrap_or_default(),
                currency_code: rec
                    .get("currency_id__ref")
                    .map(|r| strip_module_prefix(r).to_string()),
                vat_label: rec.get("vat_label").cloned(),
            })
        })
        .collect();

    write_json(&entries, dest_dir, "erp_countries.json")
}

// ---------------------------------------------------------------------------
// 2. States / Provinces (CSV)
// ---------------------------------------------------------------------------

fn download_and_process_erp_states(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}odoo/addons/base/data/res.country.state.csv");
    let data = download(client, &url)?;
    let text = String::from_utf8_lossy(&data);

    let mut entries: Vec<ErpStateEntry> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| DatjitError::Corpus(format!("read state CSV headers: {e}")))?
        .clone();

    // Find column indices
    let country_idx = headers.iter().position(|h| h.starts_with("country_id"));
    let name_idx = headers.iter().position(|h| h == "name");
    let code_idx = headers.iter().position(|h| h == "code");

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let country_ref = country_idx.and_then(|i| record.get(i)).unwrap_or("").trim();
        let name = name_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        let code = code_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();

        if name.is_empty() {
            continue;
        }

        // country_ref is like "base.us" → strip prefix, uppercase
        let country_code = strip_module_prefix(country_ref).to_uppercase();

        entries.push(ErpStateEntry {
            name,
            code,
            country_code,
        });
    }

    write_json(&entries, dest_dir, "erp_states.json")
}

// ---------------------------------------------------------------------------
// 3. Currencies
// ---------------------------------------------------------------------------

fn download_and_process_erp_currencies(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}odoo/addons/base/data/res_currency_data.xml");
    let data = download(client, &url)?;
    let xml = String::from_utf8_lossy(&data);

    let records = parse_odoo_xml_records(&xml, "res.currency");
    let entries: Vec<ErpCurrencyEntry> = records
        .iter()
        .filter_map(|rec| {
            let id = rec.get("__id__")?;
            let code = strip_module_prefix(id).to_string();
            let name = rec.get("name").cloned().unwrap_or_else(|| code.clone());
            let symbol = rec.get("symbol").cloned().unwrap_or_else(|| code.clone());
            let rounding = rec
                .get("rounding")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.01);
            let position = rec
                .get("position")
                .cloned()
                .unwrap_or_else(|| "before".into());
            Some(ErpCurrencyEntry {
                name,
                code,
                symbol,
                rounding,
                position,
            })
        })
        .collect();

    write_json(&entries, dest_dir, "erp_currencies.json")
}

// ---------------------------------------------------------------------------
// 4. Units of Measure
// ---------------------------------------------------------------------------

fn download_and_process_erp_uom(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}addons/uom/data/uom_data.xml");
    let data = download(client, &url)?;
    let xml = String::from_utf8_lossy(&data);

    // First pass: build category id → name map
    let categories = parse_odoo_xml_records(&xml, "uom.category");
    let cat_map: HashMap<String, String> = categories
        .iter()
        .filter_map(|rec| {
            let id = rec.get("__id__")?.clone();
            let name = rec.get("name")?.clone();
            Some((id, name))
        })
        .collect();

    // Second pass: parse UoM records
    let uom_records = parse_odoo_xml_records(&xml, "uom.uom");
    let entries: Vec<ErpUomEntry> = uom_records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?.clone();
            let cat_ref = rec.get("category_id__ref")?;
            let category = cat_map
                .get(cat_ref)
                .cloned()
                .unwrap_or_else(|| strip_module_prefix(cat_ref).to_string());
            let uom_type = rec
                .get("uom_type")
                .cloned()
                .unwrap_or_else(|| "reference".into());
            let factor = rec
                .get("factor")
                .or_else(|| rec.get("factor_inv"))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(1.0);
            Some(ErpUomEntry {
                name,
                category,
                uom_type,
                factor,
            })
        })
        .collect();

    write_json(&entries, dest_dir, "erp_uom.json")
}

// ---------------------------------------------------------------------------
// 5. Payment Terms (from shared account_data.xml download)
// ---------------------------------------------------------------------------

fn process_erp_payment_terms(
    account_data: Option<&[u8]>,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = account_data
        .ok_or_else(|| DatjitError::Corpus("account_data.xml not downloaded".into()))?;
    let xml = String::from_utf8_lossy(data);

    let records = parse_odoo_xml_records(&xml, "account.payment.term");
    let entries: Vec<ErpPaymentTermEntry> = records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?.clone();
            let note = rec.get("note").cloned();
            Some(ErpPaymentTermEntry { name, note })
        })
        .collect();

    if entries.is_empty() {
        return Err(DatjitError::Corpus("no payment term records found".into()));
    }

    write_json(&entries, dest_dir, "erp_payment_terms.json")
}

// ---------------------------------------------------------------------------
// 6. Account Types (from shared account_data.xml download)
// ---------------------------------------------------------------------------

fn process_erp_account_types(
    account_data: Option<&[u8]>,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let data = account_data
        .ok_or_else(|| DatjitError::Corpus("account_data.xml not downloaded".into()))?;
    let xml = String::from_utf8_lossy(data);

    // Account types are stored as account.account.tag records in Odoo 17
    let records = parse_odoo_xml_records(&xml, "account.account.tag");
    let entries: Vec<ErpAccountTypeEntry> = records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?.clone();
            let id = rec.get("__id__").cloned().unwrap_or_default();
            let account_type = strip_module_prefix(&id).to_string();
            Some(ErpAccountTypeEntry { name, account_type })
        })
        .collect();

    if entries.is_empty() {
        return Err(DatjitError::Corpus("no account type records found".into()));
    }

    write_json(&entries, dest_dir, "erp_account_types.json")
}

// ---------------------------------------------------------------------------
// 7. Incoterms
// ---------------------------------------------------------------------------

fn download_and_process_erp_incoterms(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}addons/account/data/account_incoterms_data.xml");
    let data = download(client, &url)?;
    let xml = String::from_utf8_lossy(&data);

    let records = parse_odoo_xml_records(&xml, "account.incoterms");
    let entries: Vec<ErpIncotermEntry> = records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?.clone();
            let code = rec.get("code")?.clone();
            Some(ErpIncotermEntry { name, code })
        })
        .collect();

    if entries.is_empty() {
        return Err(DatjitError::Corpus("no incoterm records found".into()));
    }

    write_json(&entries, dest_dir, "erp_incoterms.json")
}

// ---------------------------------------------------------------------------
// 8. Tax Rates (generic COA)
// ---------------------------------------------------------------------------

fn download_and_process_erp_tax_rates(
    client: &reqwest::blocking::Client,
    dest_dir: &Path,
) -> Result<u64, DatjitError> {
    let url = format!("{ODOO_RAW_BASE}addons/l10n_generic_coa/data/account_tax_template_data.xml");
    let data = download(client, &url)?;
    let xml = String::from_utf8_lossy(&data);

    // Try both model names (varies by Odoo version)
    let mut records = parse_odoo_xml_records(&xml, "account.tax");
    if records.is_empty() {
        records = parse_odoo_xml_records(&xml, "account.tax.template");
    }

    let entries: Vec<ErpTaxRateEntry> = records
        .iter()
        .filter_map(|rec| {
            let name = rec.get("name")?.clone();
            let rate = rec
                .get("amount")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let type_tax_use = rec.get("type_tax_use").cloned();
            Some(ErpTaxRateEntry {
                name,
                rate,
                type_tax_use,
            })
        })
        .collect();

    if entries.is_empty() {
        return Err(DatjitError::Corpus("no tax rate records found".into()));
    }

    write_json(&entries, dest_dir, "erp_tax_rates.json")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_odoo_xml_records() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="base.es" model="res.country">
        <field name="name">Spain</field>
        <field name="code">ES</field>
        <field name="phone_code">34</field>
        <field name="currency_id" ref="base.EUR"/>
        <field name="vat_label">NIF</field>
    </record>
    <record id="base.fr" model="res.country">
        <field name="name">France</field>
        <field name="code">FR</field>
        <field name="phone_code">33</field>
        <field name="currency_id" ref="base.EUR"/>
    </record>
    <record id="base.other" model="res.partner">
        <field name="name">Should be skipped</field>
    </record>
</odoo>"#;

        let records = parse_odoo_xml_records(xml, "res.country");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get("name").unwrap(), "Spain");
        assert_eq!(records[0].get("code").unwrap(), "ES");
        assert_eq!(records[0].get("phone_code").unwrap(), "34");
        assert_eq!(records[0].get("currency_id__ref").unwrap(), "base.EUR");
        assert_eq!(records[0].get("vat_label").unwrap(), "NIF");
        assert_eq!(records[1].get("name").unwrap(), "France");
    }

    #[test]
    fn test_parse_odoo_xml_eval() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="base.EUR" model="res.currency">
        <field name="name">EUR</field>
        <field name="symbol">€</field>
        <field name="rounding" eval="0.01"/>
        <field name="position">before</field>
    </record>
</odoo>"#;

        let records = parse_odoo_xml_records(xml, "res.currency");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get("rounding").unwrap(), "0.01");
        assert_eq!(records[0].get("symbol").unwrap(), "€");
    }

    #[test]
    fn test_strip_module_prefix() {
        assert_eq!(strip_module_prefix("base.EUR"), "EUR");
        assert_eq!(strip_module_prefix("base.us"), "us");
        assert_eq!(
            strip_module_prefix("uom.product_uom_categ_unit"),
            "product_uom_categ_unit"
        );
        assert_eq!(strip_module_prefix("standalone"), "standalone");
    }

    #[test]
    fn test_parse_odoo_xml_self_closing_field() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="uom.product_uom_kgm" model="uom.uom">
        <field name="name">kg</field>
        <field name="category_id" ref="uom.product_uom_categ_kgm"/>
        <field name="uom_type">reference</field>
        <field name="factor" eval="1.0"/>
    </record>
</odoo>"#;

        let records = parse_odoo_xml_records(xml, "uom.uom");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get("name").unwrap(), "kg");
        assert_eq!(
            records[0].get("category_id__ref").unwrap(),
            "uom.product_uom_categ_kgm"
        );
        assert_eq!(records[0].get("factor").unwrap(), "1.0");
    }
}
