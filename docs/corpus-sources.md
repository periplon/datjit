# Datjit Corpus Sources

Complete reference for all data sources downloaded by `datjit corpus update`.

## Overview

The corpus system downloads structured data from public sources and stores it as JSON files in `~/.datjit/corpus/`. The data powers realistic synthetic data generation for all semantic types in DDL schemas.

```
~/.datjit/corpus/
  en-US/                    # Locale-specific (US English)
    person_first.json       # First names with gender + frequency
    person_last.json        # Surnames with frequency
    cities.json             # Cities worldwide with coordinates
    postal_codes.json       # US ZIP codes with city/state
  shared/                   # Language-neutral / global
    countries.json          # Countries with currencies, phone codes
    admin1.json             # States/provinces worldwide
    timezones.json          # IANA timezones
    currencies.json         # ISO 4217 currencies with symbols
    languages.json          # ISO 639 language codes
    job_titles.json         # O*NET occupations + alternate titles
    phone_formats.json      # Phone patterns per country
    credit_card_bins.json   # Card number prefixes by brand
    product_categories.json # Google product taxonomy
    products.json           # Best Buy product catalog
    airports.json           # Airports with IATA codes
    airlines.json           # Airlines with IATA/ICAO
    vehicles.json           # Vehicle makes/models
    foods.json              # USDA foundation foods
    food_products.json      # Open Food Facts products
    institutions.json       # US colleges/universities
    stock_tickers.json      # SEC company tickers
    companies.json          # Wikidata global companies
    german_companies.json   # German Handelsregister
    icd10_codes.json        # ICD-10 medical diagnosis codes
    mac_vendors.json        # IEEE OUI MAC prefixes
    tlds.json               # IANA top-level domains
    mime_types.json         # IANA media types
    locale_formats.json     # CLDR locale formatting rules
```

## Source Details

### Person Names

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `en-US/person_first.json` | US Census Bureau 1990 First Names | Public Domain | 5,494 | Gender + frequency weight per name |
| `en-US/person_last.json` | US Census Bureau 2010 Surnames | Public Domain | 5,000 | Frequency weight (per 100K population) |

**Format:**
```json
[{"name": "James", "weight": 3.318, "gender": "male"}, ...]
[{"name": "Smith", "weight": 2442.0, "gender": null}, ...]
```

**URLs:**
- `https://www2.census.gov/topics/genealogy/1990surnames/dist.female.first`
- `https://www2.census.gov/topics/genealogy/1990surnames/dist.male.first`
- `https://www2.census.gov/topics/genealogy/2010surnames/names.zip`

---

### Geography & Addresses

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `en-US/cities.json` | GeoNames cities15000 | CC BY 4.0 | 33,473 | Population > 15K, worldwide, with timezone |
| `en-US/postal_codes.json` | GeoNames US postal codes | CC BY 4.0 | 41,490 | ZIP -> city -> state -> lat/lng |
| `shared/countries.json` | GeoNames countryInfo | CC BY 4.0 | 252 | ISO codes, capital, currency, phone prefix |
| `shared/admin1.json` | GeoNames admin1Codes | CC BY 4.0 | 3,862 | States/provinces per country |
| `shared/timezones.json` | GeoNames timeZones | CC BY 4.0 | 418 | IANA timezone + GMT offset + country |

**City entry format:**
```json
{"name": "New York", "ascii_name": "New York", "lat": 40.7143, "lng": -74.006,
 "country": "US", "admin1": "NY", "population": 8804190, "timezone": "America/New_York"}
```

**Country entry format:**
```json
{"code": "US", "iso3": "USA", "name": "United States", "capital": "Washington",
 "population": 327167434, "continent": "NA", "currency_code": "USD",
 "currency_name": "Dollar", "phone_prefix": "1", "languages": "en-US,es-US,haw,fr"}
```

**URLs:**
- `https://download.geonames.org/export/dump/cities15000.zip`
- `https://download.geonames.org/export/zip/US.zip`
- `https://download.geonames.org/export/dump/countryInfo.txt`
- `https://download.geonames.org/export/dump/admin1CodesASCII.txt`
- `https://download.geonames.org/export/dump/timeZones.txt`

---

### Jobs & Employment

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/job_titles.json` | O*NET Database (US DoL) | CC BY 4.0 | 1,016 occupations + 55,024 alt titles | SOC codes, descriptions, job zones (1-5) |

**Format:**
```json
{"occupations": [
  {"soc_code": "15-1252.00", "title": "Software Developers", "description": "...", "zone": 4}
], "alternate_titles": [
  {"soc_code": "15-1252.00", "title": "Full Stack Developer"}
]}
```

**URLs:**
- `https://www.onetcenter.org/dl_files/database/db_28_3_text/Occupation%20Data.txt`
- `https://www.onetcenter.org/dl_files/database/db_28_3_text/Alternate%20Titles.txt`
- `https://www.onetcenter.org/dl_files/database/db_28_3_text/Job%20Zones.txt`

---

### Companies & Commerce

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/stock_tickers.json` | SEC EDGAR | Public Domain | 10,433 | Ticker, company name, exchange (NYSE/Nasdaq/OTC) |
| `shared/companies.json` | Wikidata SPARQL | CC0 | ~6,800 | Global companies with country + industry |
| `shared/german_companies.json` | OffeneRegister.de | CC0 | 10,000 | German company register (streamed from 773 MB bz2) |

**Stock ticker format:**
```json
{"ticker": "AAPL", "name": "APPLE INC", "cik": 320193, "exchange": "Nasdaq"}
```

**URLs:**
- `https://www.sec.gov/files/company_tickers_exchange.json` (requires UA with email)
- `https://query.wikidata.org/sparql` (SPARQL query for Q4830453 instances)
- `https://daten.offeneregister.de/de_companies_ocdata.jsonl.bz2`

---

### Products & Food

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/products.json` | Best Buy Open Dataset | Public Domain | 51,645 | Name, SKU, price, manufacturer, category |
| `shared/product_categories.json` | Google Product Taxonomy | CC BY 4.0 | 5,595 | Hierarchical category tree |
| `shared/foods.json` | USDA FoodData Central | Public Domain | 387 | Foundation food names with categories |
| `shared/food_products.json` | Open Food Facts | ODbL | 50,000 | Branded food products (streamed from 1.2 GB gzip) |

**Product format:**
```json
{"name": "Duracell - AAA Batteries (4-Pack)", "sku": "43900", "price": 5.49,
 "manufacturer": "Duracell", "category": "Alkaline Batteries",
 "description": "Compatible with select electronic devices..."}
```

**URLs:**
- `https://raw.githubusercontent.com/BestBuyAPIs/open-data-set/master/products.json`
- `https://www.google.com/basepages/producttype/taxonomy-with-ids.en-US.txt`
- `https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_foundation_food_csv_2024-10-31.zip`
- `https://static.openfoodfacts.org/data/en.openfoodfacts.org.products.csv.gz`

---

### Transportation

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/airports.json` | OurAirports | Public Domain | 4,557 | Large + medium airports with IATA codes |
| `shared/airlines.json` | OpenFlights | ODbL | 1,013 | Active airlines with IATA/ICAO codes |
| `shared/vehicles.json` | EPA Fuel Economy | Public Domain | 5,000 | Make, model, year, fuel type (2015+) |

**Airport format:**
```json
{"iata_code": "JFK", "name": "John F Kennedy Intl", "city": "New York",
 "country": "US", "lat": 40.6398, "lng": -73.7789, "airport_type": "large_airport"}
```

**URLs:**
- `https://davidmegginson.github.io/ourairports-data/airports.csv`
- `https://raw.githubusercontent.com/jpatokal/openflights/master/data/airlines.dat`
- `https://www.fueleconomy.gov/feg/epadata/vehicles.csv.zip`

---

### Finance & Identifiers

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/currencies.json` | CLDR + ISO 4217 (Six Group) | Unicode ToU + Free | 178 | Code, name, symbol, decimal digits |
| `shared/credit_card_bins.json` | BIN List (GitHub) | Public | 1,000 | Card prefix -> brand (Visa, MC, Amex, Discover) |

**Currency format:**
```json
{"code": "USD", "name": "US Dollar", "symbol": "$", "decimal_digits": 2}
```

**URLs:**
- CLDR: `https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/currencyData.json`
- ISO 4217: `https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml`
- BINs: `https://raw.githubusercontent.com/iannuttall/binlist-data/master/binlist-data.csv`

---

### Technical & Network

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/phone_formats.json` | Google libphonenumber | Apache 2.0 | 254 | Country phone patterns + example numbers |
| `shared/tlds.json` | IANA | Public Domain | 1,436 | All top-level domains |
| `shared/mime_types.json` | IANA Media Types | Public Domain | 2,264 | Registered MIME types by category |
| `shared/mac_vendors.json` | IEEE OUI via maclookup.app | Free | 5,000 | MAC address prefix -> vendor name |

**URLs:**
- `https://raw.githubusercontent.com/google/libphonenumber/master/resources/PhoneNumberMetadata.xml`
- `https://data.iana.org/TLD/tlds-alpha-by-domain.txt`
- `https://www.iana.org/assignments/media-types/media-types.xml`
- `https://maclookup.app/downloads/csv-database/get-db`

---

### Health & Education

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/icd10_codes.json` | CMS (US) | Public Domain | 74,260 | ICD-10 diagnosis codes + descriptions |
| `shared/institutions.json` | IPEDS (NCES) | Public Domain | 6,163 | US colleges/universities with city, state, type |

**ICD-10 format:**
```json
{"code": "A00.0", "description": "Cholera due to Vibrio cholerae 01, biovar cholerae", "category": "A00"}
```

**URLs:**
- `https://www.cms.gov/files/zip/2025-code-descriptions-tabular-order.zip`
- `https://nces.ed.gov/ipeds/datacenter/data/HD2023.zip`

---

### Locale & Internationalization

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/languages.json` | Library of Congress ISO 639-2 | Public Domain | 183 | Language code, alpha3, English name |
| `shared/locale_formats.json` | Unicode CLDR | Unicode ToU | 151 | First day of week per territory |

**URLs:**
- `https://www.loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt`
- `https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-core/supplemental/weekData.json`

---

### GitHub-Hosted Open Data (Batch 4)

| File | Source | License | Records | Notes |
|------|--------|---------|---------|-------|
| `shared/international_names.json` | sigpwned/popular-names-by-country-dataset | CC0 | ~106K | Forenames + surnames from 106 countries |
| `shared/color_names.json` | bahamas10/css-color-names | MIT | 148 | CSS named colors with hex values |
| `shared/vehicles.json` | abhionlyone/us-car-models-data | MIT | ~15K | US car make/model/year 1992-2023 |
| `shared/books.json` | zygmuntz/goodbooks-10k | CC BY-SA 4.0 | 10,000 | Book titles with authors and language |
| `shared/cryptocurrencies.json` | crypti/cryptocurrencies | MIT | ~12K | Crypto symbols and names |
| `shared/company_designators.json` | ProfoundNetworks/company_designator | CC BY-SA 3.0 | ~300 | International corporate entity suffixes (GmbH, Ltd, Inc, S.A., etc.) |
| `shared/species.json` | species-names/dataset | CC0 | varies | Scientific and common species names (birds, mammals, reptiles) |

**International names format:**
```json
{
  "forenames": [{"name": "Maria", "country": "ES", "gender": "F"}, ...],
  "surnames": [{"name": "Garcia", "country": "ES", "gender": null}, ...]
}
```

**Book format:**
```json
{"title": "The Hunger Games", "authors": "Suzanne Collins", "language": "eng"}
```

**Company designator format:**
```json
{"designator": "GmbH", "country": "DE"}
```

**URLs:**
- `https://raw.githubusercontent.com/sigpwned/popular-names-by-country-dataset/main/forenames.csv`
- `https://raw.githubusercontent.com/sigpwned/popular-names-by-country-dataset/main/surnames.csv`
- `https://raw.githubusercontent.com/bahamas10/css-color-names/master/css-color-names.json`
- `https://raw.githubusercontent.com/abhionlyone/us-car-models-data/master/cars_data.csv`
- `https://raw.githubusercontent.com/zygmuntz/goodbooks-10k/master/books.csv`
- `https://raw.githubusercontent.com/crypti/cryptocurrencies/master/cryptocurrencies.json`
- `https://raw.githubusercontent.com/ProfoundNetworks/company_designator/master/company_designator/data/company_designator.csv`
- `https://raw.githubusercontent.com/species-names/dataset/main/{birds,mammals,reptiles}/index.json`

---

## Totals

| Category | Files | Records | Size |
|----------|-------|---------|------|
| Person Names | 3 | ~116K | ~1.5 MB |
| Geography | 5 | 79,495 | 12.9 MB |
| Jobs | 1 | 56,040 | 4.7 MB |
| Companies | 4 | ~27,500 | 1.9 MB |
| Products & Food | 4 | ~107,600 | 27.1 MB |
| Transportation | 3 | ~25K | 2.5 MB |
| Finance | 3 | ~13K | 0.4 MB |
| Technical | 4 | 8,954 | 0.6 MB |
| Health & Education | 2 | 80,423 | 11.8 MB |
| Locale | 2 | 334 | < 0.1 MB |
| Books & Media | 1 | 10,000 | ~1.0 MB |
| Colors | 1 | 148 | < 0.1 MB |
| Science | 1 | varies | varies |
| **Total** | **~35** | **~525,000+** | **~65 MB** |

## Usage

```bash
# Download all corpus data
datjit corpus update

# Check what's installed
datjit corpus info

# List available sources
datjit corpus list
```

The corpus is downloaded to `~/.datjit/corpus/` and used automatically by the generator when available. Without the corpus, datjit falls back to a minimal embedded dataset of ~200 names and 50 cities.

## Large Source Streaming

Two sources are streamed rather than fully downloaded:

- **Open Food Facts** (1.2 GB gzipped CSV): Decompressed and parsed on-the-fly, extracting only the first 50,000 products with non-empty ASCII-friendly names.
- **German Handelsregister** (773 MB bz2 JSONL): Decompressed and parsed line-by-line, extracting the first 10,000 company entries.

Both use streaming decompression (`flate2::GzDecoder`, `bzip2::BzDecoder`) to avoid loading the full file into memory.
