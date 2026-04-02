# DDL Corpus Sources — Supplement

Additional public data sources organized by semantic category. All free, all downloadable.

---

## 1. Internationalization & Locale Infrastructure

**Unicode CLDR (Common Locale Data Repository)**
- URL: https://cldr.unicode.org / https://github.com/unicode-org/cldr
- License: Unicode Terms of Use (permissive)
- Contents for DDL:
  - Currency symbols, names, formatting patterns for every locale
  - Number formatting (decimal separators, grouping) per locale
  - Date/time patterns per locale
  - Country names in 186+ languages
  - Language names, script names
  - Measurement systems per country (metric vs imperial)
  - First day of week per country
  - Calendar preferences per locale
  - Person name formatting order per locale (given-family vs family-given)
  - Exemplar character sets per language
- Why critical: This is the single source of truth for locale-aware formatting. When your generator outputs a date or currency value, CLDR tells you how to format it for `ja-JP` vs `de-DE`

**ISO Standards (freely published subsets)**
- ISO 639-1/639-3: Language codes — https://www.loc.gov/standards/iso639-2/php/code_list.php
- ISO 3166-1: Country codes — https://www.iso.org/obp/ui/#search (browsable), Wikipedia has comprehensive tables
- ISO 4217: Currency codes — https://www.six-group.com/en/products-services/financial-information/data-standards.html
- ISO 15924: Script codes — https://unicode.org/iso15924/iso15924-codes.html

---

## 2. Vehicles

**NHTSA vPIC (Vehicle Product Information Catalog)**
- URL: https://vpic.nhtsa.dot.gov/api/ (API) / https://vpic.nhtsa.dot.gov/ (standalone DB download)
- License: Public domain (US government)
- Contents: Every make, model, and variant sold in the US since 1981. Includes body type, engine specs, fuel type, drive type, manufacturer, plant location
- Standalone database available as SQL Server and PostgreSQL dumps
- Use for: `vehicle.make`, `vehicle.model`, `vehicle.year`, `vehicle.type` semantic types

**EPA Fuel Economy Data**
- URL: https://www.fueleconomy.gov/feg/download.shtml
- License: Public domain
- Contents: Every vehicle model year since 1984 with MPG ratings, engine displacement, cylinders, transmission type, fuel type, CO2 emissions
- Format: CSV/XML
- Use for: Correlated vehicle attributes (a 2024 Tesla Model 3 → electric, no cylinders, specific MPG-equivalent)

---

## 3. Education

**IPEDS (Integrated Postsecondary Education Data System)**
- URL: https://nces.ed.gov/ipeds/use-the-data
- License: Public domain
- Contents: 7,000+ US colleges and universities with: institution name, address, type (public/private/for-profit), Carnegie classification, enrollment, tuition, graduation rates, degrees conferred by field
- Format: CSV, Access database
- Use for: `education.institution`, `education.degree`, realistic university names with coherent location

**CIP Codes (Classification of Instructional Programs)**
- URL: https://nces.ed.gov/ipeds/cipcode/
- Contents: Hierarchical taxonomy of 1,500+ academic programs (e.g., "11.0701 Computer Science", "52.0201 Business Administration")
- Use for: `education.field`, `education.major` semantic types

**World University Rankings (multiple sources)**
- QS, THE, ARWU all publish downloadable lists
- Use for international institution name corpora

---

## 4. Food & Agriculture

**USDA FoodData Central**
- URL: https://fdc.nal.usda.gov/download-datasets
- License: Public domain
- Contents: 300,000+ food items with names, categories, nutrients, portions, branded products
- Format: CSV/JSON
- Use for: `food.name`, `food.category`, realistic grocery/menu item names

**Open Food Facts**
- URL: https://world.openfoodfacts.org/data
- License: ODbL (Open Database License)
- Contents: 3M+ food products worldwide with name, brand, category, barcode, nutrition, ingredients, country
- Format: CSV, MongoDB dump
- Use for: Multi-locale food product names, brand names, barcodes (EAN-13)

---

## 5. Medical & Health

**ICD-10 Codes**
- URL: https://www.cms.gov/medicare/coding-billing/icd-10-codes
- License: Public domain (US government)
- Contents: 70,000+ diagnosis codes with descriptions
- Use for: `medical.diagnosis`, healthcare domain test data

**NDC (National Drug Code) Directory**
- URL: https://www.fda.gov/drugs/drug-approvals-and-databases/national-drug-code-directory
- License: Public domain
- Contents: All FDA-registered drugs with proprietary name, generic name, dosage form, route, strength, manufacturer
- Format: CSV
- Use for: `drug.name`, `drug.generic`, pharmaceutical domain data

**NPI (National Provider Identifier) Registry**
- URL: https://download.cms.gov/nppes/NPI_Files.html
- License: Public domain
- Contents: 8M+ healthcare provider records with name, specialty, address, organization
- Use for: Doctor/provider names, medical specialties, healthcare org names

**SNOMED CT (via UMLS)**
- URL: https://www.nlm.nih.gov/research/umls/
- License: Free with UMLS license (registration required)
- Contents: Comprehensive medical terminology — procedures, anatomy, diseases

---

## 6. Transportation & Travel

**Airport Codes (OurAirports)**
- URL: https://ourairports.com/data/
- License: Public domain
- Contents: 55,000+ airports worldwide with IATA/ICAO codes, name, city, country, lat/lng, elevation, type
- Format: CSV
- Use for: `airport.iata`, `airport.name`, travel domain data

**Airline Data (OpenFlights)**
- URL: https://openflights.org/data
- License: ODbL
- Contents: 6,000+ airlines (name, IATA/ICAO code, country, active status), 67,000+ routes, 7,000+ airports
- Format: CSV
- Use for: `airline.name`, `airline.iata`, flight/booking domain

**UN/LOCODE (Location Codes)**
- URL: https://unece.org/trade/uncefact/unlocode
- License: Free
- Contents: 100,000+ location codes for ports, airports, inland freight locations worldwide
- Use for: Logistics/shipping domain data

---

## 7. Network & Technical

**IEEE OUI (MAC Address Vendor Prefixes)**
- URL: https://maclookup.app/downloads/csv-database
- Also: https://standards-oui.ieee.org/ (official source)
- License: Free to use
- Contents: 57,000+ OUI → vendor mappings (prefix, company name, address, country)
- Format: CSV, JSON, XML
- Use for: `mac` semantic type — generate valid-looking MACs with real vendor prefixes

**IANA Registries**
- Root Zone Database (TLDs): https://www.iana.org/domains/root/db
  - All top-level domains (.com, .org, .uk, .jp, etc.)
- Service Name and Transport Protocol Port Numbers: https://www.iana.org/assignments/service-names-port-numbers/
  - Every assigned port number with service name and description
- Media Types (MIME): https://www.iana.org/assignments/media-types/
  - All registered MIME types
- Time Zone Database: https://www.iana.org/time-zones
  - Definitive timezone list with historical rules
- Use for: `url`, `file.mime`, `port`, `timezone` semantic types

**Public Suffix List**
- URL: https://publicsuffix.org/list/
- License: MPL 2.0
- Contents: All effective TLDs including ccTLDs and private domains (e.g., `.co.uk`, `.com.au`, `.github.io`)
- Use for: Generating realistic domain names and URLs

**User-Agent Strings**
- URL: https://github.com/AioCrawler/ua-datasets or various UA databases
- Contents: Real browser/OS user agent strings with market share
- Use for: Web/analytics domain test data

---

## 8. Government & Administrative

**FIPS Codes (US Federal Information Processing Standards)**
- URL: https://www.census.gov/library/reference/code-lists/ansi.html
- Contents: State codes (2-digit), county codes (5-digit), place codes for every US jurisdiction
- Use for: Government/administrative domain, coherent state → county → place hierarchies

**Country Subdivisions (ISO 3166-2)**
- URL: Wikipedia maintains comprehensive tables; also available via CLDR
- Contents: All states/provinces/regions for every country with standard codes
- Use for: International address generation — `address.state` equivalents per country

**US ZIP Code Database**
- URL: https://www.unitedstateszipcodes.org/zip-code-database/ (free version)
- Also: GeoNames postal codes, TIGER/Line
- Contents: ZIP → city, state, county, lat/lng, population, area
- Use for: The coherence index — picking a ZIP and deriving everything else

---

## 9. Commerce & Business

**NAICS Codes (North American Industry Classification System)**
- URL: https://www.census.gov/naics/
- License: Public domain
- Contents: Hierarchical industry classification — 20 sectors, ~1,000 detailed industries
- Format: Excel/CSV
- Use for: `company.industry`, business domain classification

**SIC Codes (Standard Industrial Classification)**
- URL: https://www.osha.gov/data/sic-manual
- Contents: Older but still widely used industry classification, ~1,000 codes
- Use for: Legacy business domain data

**UN UNSPSC (Standard Products and Services Code)**
- URL: https://www.unspsc.org/ (free browse, registration for download)
- Contents: 50,000+ product/service categories in a 4-level hierarchy
- Use for: `product.category` at fine granularity

**Stock Tickers**
- SEC EDGAR Company Tickers: https://www.sec.gov/files/company_tickers.json
- License: Public domain
- Contents: All publicly traded US companies with ticker, name, CIK
- Use for: `stock.ticker`, `company.name` (public companies)

---

## 10. Scientific & Reference

**Chemical Elements**
- URL: IUPAC or Wikipedia periodic table data
- Contents: 118 elements with symbol, name, atomic number, atomic weight, category
- Use for: Science domain test data

**Country Facts (World Bank Open Data)**
- URL: https://data.worldbank.org/
- License: CC BY 4.0
- Contents: GDP, population, literacy rate, life expectancy, and 1,400+ indicators for every country
- Use for: Enriching country records with realistic economic/demographic data

**Wikidata**
- URL: https://www.wikidata.org/wiki/Wikidata:Database_download
- License: CC0
- Contents: Structured data for 100M+ entities — people, places, organizations, works, species
- Use for: Any domain where you need large, diverse, real-world entity names and attributes. Query via SPARQL for specific entity types.

---

## 11. Names (International Supplements)

**Behind the Name (Database)**
- URL: https://www.behindthename.com/
- Contents: 30,000+ given names with origin, gender, usage by country, popularity rankings
- Not freely downloadable in bulk, but structured and scrapeable for research

**Forebears Surname Distribution**
- URL: https://forebears.io/surnames
- Contents: Surname frequency and geographic distribution for 200+ countries
- Use for: International surname corpus where census data isn't available

**Wikipedia "Most Common Given Names" / "Most Common Surnames"**
- URL: Various per-country Wikipedia articles
- Contents: Community-maintained lists with frequency data sourced from national statistics
- Use for: Quick bootstrap for locales without government open data

---

## 12. Images & Avatars (Placeholder)

**DiceBear**
- URL: https://www.dicebear.com/
- License: MIT
- Contents: Deterministic avatar generation from seed strings — multiple art styles
- Use for: `person.avatar` — generate a consistent avatar URL from a person's name/ID

**UI Faces / This Person Does Not Exist**
- URL: https://uifaces.co/ / https://thispersondoesnotexist.com/
- Use for: Realistic face photo URLs for test data (generated, not real people)

**Picsum Photos**
- URL: https://picsum.photos/
- License: Free
- Contents: Deterministic placeholder images by seed, size, and effect
- Use for: `url.image` semantic type — `https://picsum.photos/seed/{entity_id}/400/300`

---

## 13. Miscellaneous Lookup Tables

| What | Source | URL |
|---|---|---|
| US state abbreviations | Census | https://www.census.gov/library/reference/code-lists/ansi.html |
| Country calling codes | ITU-T E.164 | Wikipedia's comprehensive table |
| Credit card BIN ranges | Open BIN databases | https://github.com/iannuttall/binlist-data |
| License plate formats | Per-country DOT sources | Wikipedia per-country articles |
| Vehicle colors | BASF Color Report | Published annually, top 10 colors by region |
| Blood types | Distribution by country | Academic/WHO statistics |
| Pet breeds | AKC (dogs), CFA (cats) | https://www.akc.org/dog-breeds/ |
| Sports teams | Wikipedia league articles | Structured tables with city, name, conference |
| Book ISBNs | Open Library | https://openlibrary.org/developers/dumps |
| Movie/TV titles | TMDb | https://www.themoviedb.org/documentation/api (API, free) |
| Music genres | Musicbrainz | https://musicbrainz.org/doc/Genre |
| HTTP status codes | IANA | https://www.iana.org/assignments/http-status-codes/ |
| Programming languages | GitHub Linguist | https://github.com/github-linguist/linguist |
| Emoji | Unicode | https://unicode.org/emoji/charts/full-emoji-list.html |
| Country flags | Flagpedia | https://flagpedia.net/download/api |

---

## 14. Meta-Sources (Aggregators)

**Awesome Public Datasets**
- URL: https://github.com/awesomedata/awesome-public-datasets
- Contents: Curated list of 500+ public datasets across every domain

**Data.gov (US)**
- URL: https://data.gov
- Contents: 300,000+ datasets from US federal agencies

**EU Open Data Portal**
- URL: https://data.europa.eu/
- Contents: 1.7M+ datasets from EU institutions and member states

**Kaggle Datasets**
- URL: https://www.kaggle.com/datasets
- Contents: 300,000+ community-contributed datasets, many with cleaned/structured data ready for corpus extraction

---

## 15. Corpus Priority Matrix

Not all sources are equal. Here's what to grab first vs what's nice-to-have:

### Tier 1 — Core (covers 80% of semantic types)
| Source | Enables |
|---|---|
| SSA Baby Names | `person.first` with frequency weights |
| Census Surnames | `person.last` with frequency + ethnicity |
| GeoNames cities + postal codes | `address.*`, `geo.*`, `timezone`, coherence index |
| Faker.js locale data | Bootstrap for all 70+ locales |
| CLDR | All locale formatting rules |
| libphonenumber | `phone` patterns per country |
| O*NET + BLS | `job.title`, `job.department`, salary distributions |

### Tier 2 — Domain-Specific (add as needed)
| Source | Enables |
|---|---|
| OpenAddresses | `address.full` with real street-level data |
| NHTSA vPIC | `vehicle.*` |
| USDA FoodData | `food.*` |
| IPEDS | `education.*` |
| OurAirports + OpenFlights | `airport.*`, `airline.*` |
| SEC tickers | `stock.ticker`, public `company.name` |

### Tier 3 — Enrichment (polish and depth)
| Source | Enables |
|---|---|
| World Bank indicators | Country-level economic coherence |
| IEEE OUI | `mac` with real vendor prefixes |
| IANA registries | `file.mime`, TLDs, ports |
| CMS ICD-10 / NDC | Healthcare domain |
| Wikidata | Any entity type at massive scale |