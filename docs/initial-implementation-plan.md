# Datjit Implementation Plan

## Context

Implement the DDL (Data Domain Language) v0.1 spec from `docs/datjit-spec.md` as a Rust CLI tool. The repo is empty (no code yet). The tool parses YAML-based DDL schemas defining data domains with semantic types, distributions, and constraints, then generates synthetic data in multiple output formats (CSV, JSON, NDJSON, YAML, SQL). A corpus system backed by real-world data sources (`docs/datjit-sources.md`) provides realistic generation. Corpus sources can be updated via CLI.

Architecture: Hexagonal (ports & adapters), SOLID, CLEAN principles. Cargo workspace with 6 crates.

---

## Directory Structure

```
datjit/
├── Cargo.toml                    # workspace root
├── .gitignore
├── crates/
│   ├── datjit-core/              # Domain: types, models, port traits, errors, Value
│   ├── datjit-parser/            # Adapter: YAML + embedded DSL parsing
│   ├── datjit-generator/         # Adapter: data generation engine
│   ├── datjit-output/            # Adapter: format writers (csv, json, yaml, sql, ndjson)
│   ├── datjit-corpus/            # Adapter: corpus management, loading, updating
│   └── datjit-cli/               # CLI entry point (clap)
├── corpus/                       # Bundled corpus data (JSON files)
│   ├── en-US/
│   └── shared/
├── tests/
│   ├── fixtures/                 # DDL YAML test fixtures
│   └── integration/
└── docs/
```

## Crate Dependency Graph

```
datjit-core: zero internal deps (thiserror, chrono, uuid, serde, indexmap)
datjit-parser: datjit-core + serde_yaml
datjit-generator: datjit-core + datjit-corpus + rand + rand_distr + chrono
datjit-output: datjit-core + csv + serde_json + serde_yaml
datjit-corpus: datjit-core + serde_json + reqwest (optional feature)
datjit-cli: all crates + clap + anyhow
```

---

## Phase 1: Workspace Scaffold + Core Domain

1. Create `Cargo.toml` workspace with all 6 crate stubs
2. Create `.gitignore` for Rust
3. **datjit-core/src/types/**: `PrimitiveType` enum (12 variants), `SemanticType` struct (namespace.tag), `CompoundType` enum (List, Map, Tuple, Nullable, Union), `ReferenceType` enum (BelongsTo, HasMany, ManyToMany, SelfRef, Polymorphic), `TypeExpr` enum unifying all
4. **datjit-core/src/model/**: `DdlDocument`, `Entity`, `Field`, `Decorator` (30+ variants), `EnumDef`, `TypeDef`, `Rule`, `VolumeSpec`, `GenerationConfig`, `CoherenceGroup`, `ToolOverride`
5. **datjit-core/src/ports/**: `DdlParser`, `DataGenerator`, `OutputWriter`, `CorpusProvider` traits
6. **datjit-core/src/error.rs**: `DatjitError` with thiserror
7. **datjit-core/src/value.rs**: `Value` enum (Null, Bool, Int, Float, String, DateTime, Date, Time, Uuid, Bytes, List, Map, Tuple, Ref)
8. Unit tests for all core types

## Phase 2: Parser

1. **yaml_parser.rs**: Deserialize YAML to `serde_yaml::Value`, walk tree to build `DdlDocument`. Header fields, entities, enums, types, rules, tools sections.
2. **type_parser.rs**: Hand-written recursive descent for field type strings. Parse order: union (`|`) > nullable (`?`) > compound (`[T]`, `{K:V}`, `(T1,T2)`) > reference (`->`, `<->`) > enum (`enum(...)`) > parameterized primitive (`int(32)`, `decimal(10,2)`) > bare primitive > semantic (`person.full`, `email`) > named type ref
3. **decorator_parser.rs**: Split field string at `@` boundaries (not inside parens), parse each decorator with args. Handle `@range(lo..hi)`, `@dist(normal, mu=X, sigma=Y)`, `@pattern("SKU-{AA}-{0000}")`, `@derived(expr)`, etc.
4. **rule_parser.rs**: Parse rule strings into `RuleExpression` (comparisons, conditionals, aggregates, unique composites)
5. **expr_parser.rs**: Parse `@derived(...)` expressions (arithmetic, function calls, field refs)
6. **validation.rs**: Entity refs exist, named types resolve, derived deps acyclic, range lo<=hi, dist percentages valid
7. Unit tests per parser: one test per type kind, decorator, rule pattern

## Phase 3: Minimal Generator + JSON Output

1. **plan.rs**: Build topological sort of entities by reference dependencies, compute volumes
2. **context.rs**: `GenerationContext` with seeded RNG, generated data store, unique sets, counters
3. **primitive_gen.rs**: Default generators for all 12 primitives
4. **field_gen.rs**: Dispatch logic (check @optional/@null_rate, @default, @derived, @pattern, then dispatch by TypeExpr)
5. **entity_gen.rs**: Row loop per entity
6. **engine.rs**: Orchestrate full pipeline
7. **json_writer.rs**: `{ "EntityName": [rows...] }` output
8. **datjit-corpus**: Hardcoded minimal embedded corpus (200 names, 100 cities, 50 company words, 20 job titles, email domains)
9. **datjit-cli**: `datjit generate <schema.yaml> -f json --seed N` command
10. Integration test: parse minimal.yaml fixture, generate, verify JSON

## Phase 4: References, Semantic Types, More Outputs

1. **reference_gen.rs**: Pick valid FK from already-generated entities, self-ref handling
2. **semantic_gen.rs**: person.*, email, address.*, phone, company.*, job.*, url, text.*, finance.*, color.*, file.*, identifiers
3. **datjit-corpus**: JSON file loader, `WeightedPicker`, locale fallback chains
4. Build en-US corpus files from Faker.js data (person_first.json, person_last.json, cities.json)
5. Enum generation with `@dist` categorical probabilities
6. **csv_writer.rs**, **yaml_writer.rs**, **ndjson_writer.rs**
7. CLI: `validate` command, `inspect` command, `--format csv|yaml|ndjson`
8. Tests: reference integrity, semantic type plausibility

## Phase 5: Distributions, Patterns, Rules, SQL

1. **distribution.rs**: normal, lognormal, exponential, zipf, geometric, bimodal, weighted (using rand_distr)
2. **pattern.rs**: Template expansion for `{A}`, `{AA}`, `{a}`, `{0}`, `{0000}`, `{####}`, `{word}`, `{WORD}`, `{uuid}`, `{seq}`
3. **decorator_apply.rs**: @range (numeric+date+relative dates like `now-90d`), @min/@max, @len, @values, @not_empty
4. **constraint.rs**: Rule enforcement with retry logic (@strict: retry up to 10x, @probability: probabilistic, @warn: log only)
5. **sql_writer.rs**: CREATE TABLE + INSERT INTO, dialect support (postgres/mysql/sqlite), proper escaping
6. CLI: `--volume` override, `--entity` filter, `--sql-dialect`, `--dry-run`
7. Tests: distribution shape verification, constraint satisfaction, SQL validity

## Phase 6: Coherence, Derivation, Advanced Features

1. **coherence.rs**: Coherence group generation (identity, location, role). Generate atomic bundles before independent fields
2. **derived_gen.rs**: Expression evaluator for @derived (concat, sum, count, avg, min, max, years_since, days_between, if, round, lower, upper, slug)
3. **coherent.rs** in corpus: Location bundles (city->state->zip->tz->area_code)
4. Compound type generation: List, Map, Tuple, Union
5. @from decorator: derive email from name, timezone from office
6. @correlated: correlated numeric fields
7. @after/@before/@within: temporal ordering (generate earlier field first, constrain later)
8. Entity-level decorators: @timestamps (auto-add created_at/updated_at), @soft_delete (add deleted_at), @versioned
9. Tests: coherence verification, derived field correctness

## Phase 7: Corpus Management + Polish

1. **updater.rs**: Download corpus from remote sources (SSA names, Census surnames, GeoNames cities, Faker.js locales)
2. CLI: `datjit corpus update [--locale] [--category]`, `datjit corpus list`, `datjit corpus info`
3. Tool inference engine: auto-generate CRUD tool specs from entities per spec section 9
4. Polymorphic references, self-referential trees with depth control
5. Multi-locale with fallback chains
6. Performance: parallel entity generation for independent entities
7. Error messages with line numbers and suggestions

---

## CLI Commands

```
datjit generate <schema.yaml> [-o path] [-f csv|json|ndjson|yaml|sql] [--seed N]
       [--locale bcp47] [--volume Entity=N,...] [--entity name] [--split]
       [--sql-dialect postgres|mysql|sqlite] [--pretty] [--dry-run]

datjit validate <schema.yaml>

datjit inspect <schema.yaml>

datjit corpus update [--locale bcp47] [--category cat]
datjit corpus list
datjit corpus info
```

---

## Key Design Decisions

- **Why not `#[derive(Deserialize)]`?** Field definitions like `price: currency.usd @range(1..5000)` are embedded DSL strings. Must use `serde_yaml::Value` + custom parsers.
- **Why `IndexMap`?** Definition order matters for CSV column order and `@from` field references.
- **Why separate parser crate?** Enables alternative frontends (JSON schema input) via the `DdlParser` port.
- **Why embedded minimal corpus?** Zero-config operation. Richer corpus downloaded on demand.
- **Uniqueness**: HashSet per (entity, field), retry up to 100 attempts, early error for low-cardinality fields.
- **Rules**: Check after each row. @strict: retry 10x then error. Temporal rules: generate earlier field first.

---

## Verification

1. `cargo build` compiles all crates
2. `cargo test` passes all unit + integration tests
3. `datjit generate tests/fixtures/minimal.yaml` produces valid JSON
4. `datjit generate tests/fixtures/project_management.yaml -f csv --split` produces per-entity CSV files
5. `datjit generate tests/fixtures/project_management.yaml -f sql --sql-dialect sqlite | sqlite3 test.db` loads without error
6. `datjit generate schema.yaml --seed 42` produces identical output on repeated runs
7. `datjit validate tests/fixtures/invalid.yaml` exits with meaningful error messages
8. `datjit corpus update --locale en-US` downloads and stores corpus data
9. Generated data satisfies all @strict rules, @unique constraints, @range bounds, referential integrity
