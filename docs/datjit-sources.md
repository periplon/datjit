# DDL Corpus Source Guide

Where to get real data for every semantic type in the DDL spec, organized by category. Each source is public, downloadable, and includes frequency/distribution data where possible.

---

## 1. Person Names

### First Names (Given Names)

**US Social Security Administration Baby Names**
- URL: https://www.ssa.gov/oact/babynames/limits.html
- License: CC0 (public domain)
- Contents: Every first name given to 5+ babies per year, 1880–2024. ~110K unique name/gender pairs with annual frequency counts
- Format: CSV per year (`yob2024.txt`): name, sex, count
- Also available by state: https://catalog.data.gov/dataset/baby-names-from-social-security-card-applications-state-and-district-of-columbia-data
- Why it's good: Real frequency weights, gender association, temporal trends. You can weight by recent decades for contemporary-sounding names

**UK Office for National Statistics**
- URL: https://www.ons.gov.uk/peoplepopulationandcommunity/birthsdeathsandmarriages/livebirths/datasets/babynamesenglandandwalesbabynamesstatisticsboys (and girls equivalent)
- License: Open Government Licence
- Contents: Baby names by frequency for England & Wales

**INSEE (France)**
- URL: https://www.insee.fr/fr/statistiques/2540004
- Contents: French first names by year and department since 1900

**ISTAT (Italy)**
- URL: https://www.istat.it/en/analysis-and-products/interactive-tools/baby-names
- Contents: Italian first names by frequency

**Statistics Japan**
- Annual surveys of popular baby names by kanji/reading

**Faker.js locale files (meta-source)**
- URL: https://github.com/faker-js/faker/tree/main/src/locales
- License: MIT
- Contents: Pre-curated name lists for 70+ locales, already structured as arrays
- Why it's good: Fastest path to multi-locale coverage. Lists are small (~200–1000 names per locale) but hand-curated for realism. Use as a bootstrap, then enrich with government sources

### Last Names (Surnames)

**US Census Bureau Surname Files**
- URL: https://www.census.gov/topics/population/genealogy/data/2010_surnames.html
- License: Public domain (US government work)
- Contents: 162,253 surnames occurring 100+ times in the 2010 Census, with frequency, rank, and race/ethnicity distribution percentages
- Format: Excel/CSV. Fields: name, rank, count, proportion per 100K, cumulative proportion, % White, % Black, % Asian, % Hispanic, etc.
- Why it's good: Real population frequency AND ethnicity correlations — enables coherent generation where surname ethnicity matches first name origin

**1990 Census Names** (simpler, lighter)
- URL: https://www.census.gov/topics/population/genealogy/data/1990_census/1990_census_namefiles.html
- Contents: dist.all.last, dist.male.first, dist.female.first with frequency and cumulative frequency
- Good for quick bootstrapping

**International surname sources:**
- UK: ONS publishes surname frequency data
- Germany: Digitales Familiennamenwörterbuch Deutschlands (DFD) project
- Japan: 名字由来net or academic surname frequency studies
- China: 2020 Census surname frequency report (top 6000 surnames covering 99.7% of population)
- Faker.js locales contain curated surname lists per country

---

## 2. Addresses & Geography

### Full Addresses

**OpenAddresses**
- URL: https://openaddresses.io / https://results.openaddresses.io
- License: CC0 for the collection; individual sources vary (most require attribution)
- Contents: 578M+ real addresses worldwide with coordinates. Street number, street name, city, region, postcode, lat/lng
- Format: CSV files by country/region
- Coverage: Strong in US, Canada, Australia, Europe. Sparser elsewhere
- Why it's good: Real, valid, geocoded addresses. Sample randomly for realistic test data

**US Census TIGER/Line**
- URL: https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html
- License: Public domain
- Contents: Street geometries with address ranges for entire US. Street names, ZIP codes, county/state FIPS codes
- Use for: Generating valid US street names and address ranges per ZIP code

### Cities, States, Countries

**GeoNames**
- URL: https://www.geonames.org/export/ / https://download.geonames.org/export/dump/
- License: CC BY 4.0
- Contents: 11.8M+ features, 25M+ names. For cities: name, coordinates, population, country, admin divisions, timezone, elevation
- Key files:
  - `cities500.zip` — all cities with population > 500 (~200K entries)
  - `cities1000.zip` — population > 1000 (~140K)  
  - `cities5000.zip` — population > 5000 (~50K)
  - `cities15000.zip` — population > 15000 (~25K)
  - `allCountries.zip` — everything
  - `countryInfo.txt` — country metadata (capital, area, population, currency, languages, phone code)
  - `admin1CodesASCII.txt` — states/provinces
  - `alternateNamesV2.zip` — place names in multiple languages
- Why it's good: Population weights let you bias toward larger cities (realistic), timezone and country linkage enables coherence groups, multi-language names support localization

**GeoNames Postal Codes**
- URL: https://download.geonames.org/export/zip/
- Contents: Postal/ZIP codes for 100+ countries with lat/lng, place name, admin divisions
- Use for: Valid ZIP ↔ city ↔ state mappings

**Natural Earth**
- URL: https://www.naturalearthdata.com/downloads/
- License: Public domain
- Contents: Country polygons, populated places, admin boundaries at 1:10m, 1:50m, 1:110m scales
- Use for: Country boundaries, capital cities, admin region hierarchies

### Coherence Mapping (city → state → zip → timezone → phone area code)

Build a lookup table from GeoNames + TIGER:
```
{city: "Springfield", state: "IL", zip: "62704", 
 timezone: "America/Chicago", lat: 39.78, lng: -89.65,
 area_code: "217", country: "US"}
```
This single row drives coherent generation of `address.*`, `geo.*`, `timezone`, and `phone` fields simultaneously.

---

## 3. Job Titles & Organizational Data

### Occupations

**O*NET Database (US Department of Labor)**
- URL: https://www.onetcenter.org/database.html
- License: CC BY 4.0
- Contents: 
  - 1,016 occupation codes with titles and descriptions
  - 55,000+ alternate/lay titles mapped to occupation codes
  - Salary ranges, education requirements, skills, work activities
  - Job zone levels (entry → expert)
- Key files:
  - `Occupation Data.txt` — SOC codes, titles, descriptions
  - `Alternate Titles.txt` — 43,000+ alternate titles per occupation
  - `Education, Training, and Experience.txt` — required education level
  - Use with BLS wage data for salary correlation
- Why it's good: Real job titles that humans actually use, pre-mapped to standard classifications. Alternate titles give you variety ("Software Developer" → "Coder", "Programmer", "Application Developer", "Full Stack Engineer")

**BLS Occupational Employment and Wage Statistics (OEWS)**
- URL: https://www.bls.gov/oes/tables.htm
- License: Public domain
- Contents: Employment counts and wage percentiles (10th, 25th, 50th, 75th, 90th) for ~830 occupations nationally and by state/metro
- Use for: `@range` and `@dist` on salary fields correlated to job title

**Combining O*NET + BLS for coherent generation:**
```
title: "Software Developer" (from O*NET alternate titles)
department: "Engineering" (from O*NET work context)
level: "mid" (from O*NET job zone 4)
salary: $98,000 (sampled from BLS p25–p75 for SOC 15-1252)
```

### Company Names & Industries

**NAICS (North American Industry Classification System)**
- URL: https://www.census.gov/naics/
- Contents: Hierarchical industry codes with titles and descriptions (2-digit sector → 6-digit industry)
- Use for: `company.industry` semantic type

**SEC EDGAR Company Filings**
- URL: https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&type=10-K&dateb=&owner=include&count=40
- Contents: Real company names from public filings
- Use for: Enriching company name corpus (but most generators use template combiners)

**Company name generation (template approach):**
Rather than sourcing real company names (trademark issues), build a combiner:
```
prefixes: [Apex, Meridian, Nova, Atlas, Zenith, Vertex, Pulse, ...]
cores: [Tech, Data, Systems, Solutions, Digital, Cloud, ...]  
suffixes: [Inc., Corp., LLC, Labs, Group, Co., Partners, ...]
patterns: ["{prefix} {core}", "{prefix} {suffix}", "{core} {suffix}"]
```
Source word lists from: Faker.js company module, thesaurus entries for business/technology terms

---

## 4. Financial Data

### Currency & Pricing

**Bureau of Labor Statistics Consumer Price Index**
- URL: https://www.bls.gov/cpi/data.htm
- Use for: Realistic price ranges by product category

**Real pricing distributions:**
For `currency.usd @range(X..Y) @dist(lognormal)`, you don't need a corpus — you need distribution parameters. Source these from:
- Academic papers on retail price distributions (typically lognormal with μ=2.5–4.0, σ=0.8–1.5 depending on category)
- Amazon product price scrapes (aggregate statistics, not individual prices)

### Bank/Financial Identifiers

**IIN/BIN Ranges (Credit Card Prefixes)**
- URL: https://github.com/iannuttall/binlist-data or similar open BIN databases
- Contents: Card number prefix → issuer, brand (Visa, MC, Amex), country
- Use for: Generating valid-looking (but not real) credit card numbers with correct Luhn check digits

**IBAN Structure**
- URL: https://www.swift.com/standards/data-standards/iban-international-bank-account-number (spec)
- Wikipedia IBAN formats page has per-country structure
- Use for: Generating locale-correct fake IBANs (country prefix + check digits + bank code + account number, all with correct structure)

**SWIFT/BIC Codes**
- Public registries exist; Faker.js has curated lists per country

---

## 5. Text & Content

### Lorem Ipsum Alternatives

For `text.sentence`, `text.paragraph` — you don't need an LLM. You need:

**Corpus-based Markov chains** trained on:
- Project Gutenberg texts (public domain books): https://www.gutenberg.org/
- Wikipedia article dumps: https://dumps.wikimedia.org/
- Common Crawl (massive web corpus): https://commoncrawl.org/

Train a simple trigram or 4-gram Markov chain per locale. Output is grammatically plausible nonsense — perfect for test data.

**Domain-specific sentence templates:**
For `product.description`, `person.bio`, etc., build template banks:

```
product.description templates:
- "The {adj} {product} features {feature} for {benefit}."
- "Designed for {audience}, this {product} delivers {quality} performance."

person.bio templates:
- "{title} at {company} with {N}+ years in {domain}."
- "Experienced {role} specializing in {specialty} and {specialty}."
```

Source adjective/feature/benefit word lists from product listing datasets or Faker.js commerce module.

### Slugs and Identifiers

No corpus needed — derive from other generated fields:
```
slug: slugify(title)  →  "wireless-bluetooth-speaker-pro"
username: template(first_name, last_name)  →  "msantos42"
email: template(first_name, last_name, domain)  →  "maria.santos@example.com"
```

---

## 6. Phone Numbers

**ITU-T E.164 + National Numbering Plans**
- Each country has a public numbering plan document specifying valid formats
- Faker.js has curated phone format strings per locale
- Google's libphonenumber: https://github.com/google/libphonenumber
  - Contains metadata for every country: valid number patterns, area code ranges, formatting rules
  - License: Apache 2.0
  - This is the definitive source — used by Android, WhatsApp, etc.
  - Extract: `PhoneNumberMetadata.xml` contains regex patterns for valid numbers per country

**US Area Codes**
- NANPA (North American Numbering Plan Administration) publishes assigned area codes
- Use for coherent generation: city → area code → phone number

---

## 7. Email Domains

No corpus needed for generation — use template approach:
```
corporate: "{first}.{last}@{company_slug}.com"
personal: [gmail.com, yahoo.com, outlook.com, hotmail.com, icloud.com, protonmail.com]
```

Weight personal domains by market share (Gmail ~30%, Outlook ~10%, etc.)
Source: Litmus email client market share reports or similar

---

## 8. Colors

**Named Colors**
- CSS named colors: 148 standard names (https://www.w3.org/TR/css-color-4/)
- X11 color names: ~750 names
- Pantone (proprietary, but names are widely published): "Cerulean", "Living Coral", etc.
- Crayola color names: 120 colors, good for consumer-friendly names

**Hex/RGB**
Generate algorithmically — no corpus needed. For realistic UI colors, constrain to reasonable saturation/lightness ranges.

---

## 9. Products

### Product Names (template approach)

Build category-specific word lists:

**Electronics**: [Wireless, Bluetooth, Smart, Pro, Ultra, Mini, Nano, Max] × [Speaker, Headphones, Charger, Hub, Camera, Display, Keyboard]

**Clothing**: [Classic, Slim, Relaxed, Vintage, Modern] × [Cotton, Denim, Linen, Wool, Silk] × [Shirt, Jacket, Pants, Dress, Sweater]

Source word lists from:
- Amazon product taxonomy: https://www.amazon.com/gp/browse.html
- Google Product Taxonomy: https://www.google.com/basepages/producttype/taxonomy-with-ids.en-US.txt (CC BY 4.0-compatible usage)
  - 6,000+ product categories in a hierarchy
  - Great for scoping `@domain()` tags

### SKU Patterns
No corpus — pure pattern generation: `"SKU-{AA}-{0000}"`

---

## 10. Dates & Times

No corpus needed. Key parameters:

**Timezone list**: IANA Time Zone Database (https://www.iana.org/time-zones)
- Definitive list of all timezones with rules
- Already built into every OS and language runtime

**Holiday/business day calendars**: For realistic `@range` constraints, holidays can matter. Python `holidays` library has per-country data.

---

## 11. The Shortcut: Faker.js as a Meta-Source

Faker.js (https://github.com/faker-js/faker) is itself the best aggregated corpus for bootstrapping:

- **70+ locales** with curated data per module
- **MIT license** — you can extract and repackage the data files
- **Modules map directly to DDL semantic types:**

| Faker module | DDL semantic type | Corpus location in repo |
|---|---|---|
| `person.firstName` | `person.first` | `src/locales/*/person/first_name.ts` |
| `person.lastName` | `person.last` | `src/locales/*/person/last_name.ts` |
| `location.street` | `address.street` | `src/locales/*/location/street_name.ts` |
| `location.city` | `address.city` | `src/locales/*/location/city_name.ts` |
| `phone.number` | `phone` | `src/locales/*/phone_number/formats.ts` |
| `company.name` | `company.name` | `src/locales/*/company/name.ts` |
| `commerce.productName` | `product.title` | `src/locales/*/commerce/product_name.ts` |
| `internet.email` | `email` | derived from person names |

**Strategy**: Use Faker.js data as your base corpus for all locales, then enrich high-priority locales (en-US, your target markets) with government statistical sources for better frequency weighting and coverage.

---

## 12. Corpus Architecture

### File Structure

```
/corpus
  /meta
    locales.json           # supported locales, fallback chains
    semantic_tags.json     # tag → corpus file mapping
  /en-US
    person_first.json      # [{name, gender, weight}, ...]
    person_last.json       # [{name, weight, ethnicity_dist}, ...]
    cities.json            # [{name, state, zip, lat, lng, pop, tz, area_code}, ...]
    streets.json           # [{name, suffix}, ...]  (from TIGER)
    job_titles.json        # [{title, soc_code, department, zone, salary_p25_p75}, ...]
    phone_formats.json     # [{pattern, area_codes}, ...]
    company_words.json     # {prefixes, cores, suffixes, patterns}
    product_words.json     # {by_domain: {electronics: {adj, noun}, ...}}
    email_domains.json     # [{domain, weight}, ...]
  /ja-JP
    person_first.json
    person_last.json
    cities.json
    ...
  /shared
    countries.json         # [{code, name, currency, phone_prefix, ...}]
    timezones.json         # from IANA
    currencies.json        # [{code, symbol, name, decimal_places}]
    colors.json            # [{name, hex}]
    iban_formats.json      # [{country, pattern, check_method}]
```

### Coherence Index

The key insight is that many corpus files need cross-references for coherent generation:

```json
// cities.json entry enables coherent address + phone + timezone
{
  "name": "Springfield",
  "state": "IL",
  "state_full": "Illinois",
  "zip_ranges": ["62701-62709", "62711-62712"],
  "lat": 39.7817,
  "lng": -89.6501,
  "population": 114394,
  "timezone": "America/Chicago",
  "area_codes": ["217"],
  "county": "Sangamon"
}
```

Picking a city first, then deriving state, ZIP, timezone, area code, and coordinates from it is how you get coherence without an LLM.

### Size Estimates

| Corpus file | Entries | Compressed size |
|---|---|---|
| US first names (SSA) | ~110K | ~1.5 MB |
| US surnames (Census) | ~162K | ~3 MB |
| Cities worldwide (GeoNames 1000+) | ~140K | ~8 MB |
| US addresses (OpenAddresses sample) | ~1M (sampled) | ~40 MB |
| Job titles (O*NET alternate) | ~43K | ~1 MB |
| Phone formats (libphonenumber) | ~250 countries | ~200 KB |
| All Faker.js locales | ~70 locales | ~5 MB |
| **Total working corpus** | | **~60 MB** |

Fits comfortably in memory. No database needed.