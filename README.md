# datjit

Synthetic data generation from declarative schemas. Define your data domain in YAML, get realistic test data in JSON, CSV, SQL, YAML, or NDJSON.

## Quick Start

```bash
cargo build --release
```

Define a schema:

```yaml
domain: my_app
seed: 42

volume:
  User: 100

entities:
  User:
    id: uuid @primary
    name: person.full
    email: email @unique
    age: int @range(18..65)
    active: bool
```

Generate data:

```bash
datjit generate schema.yaml                    # JSON to stdout
datjit generate schema.yaml -f csv -o out.csv  # CSV to file
datjit generate schema.yaml -f sql             # SQL INSERT statements
datjit generate schema.yaml --seed 42          # Deterministic output
```

## The DDL Language

DDL (Data Domain Language) is a compact YAML-based language for defining data domains. Fields combine a **type** with **decorators** that control generation.

### Types

| Category | Examples |
|----------|---------|
| Primitives | `string`, `int`, `float`, `bool`, `datetime`, `date`, `uuid`, `decimal(10,2)` |
| Semantic | `person.full`, `email`, `phone`, `address.city`, `company.name`, `job.title`, `color.hex` |
| Enums | `enum(active, inactive, suspended)` |
| References | `->User`, `->User?`, `<->Tag`, `->self?` |
| Compound | `[int]`, `{string: int}`, `string?`, `string \| int` |
| Named | `Address` (from `types:` section) |

### Decorators

```yaml
id: uuid @primary                                    # primary key
name: person.full @unique                            # all values unique
email: email @from(name)                             # derived from name
age: int @range(18..95) @dist(normal, mu=35, sigma=12)  # constrained + distributed
tier: enum(free, pro, enterprise) @dist(70, 25, 5)   # weighted enum
sku: string @pattern("SKU-{AA}-{0000}")              # template pattern
price: currency.usd @range(1..5000) @dist(lognormal) # log-normal pricing
avatar: url.avatar? @null_rate(0.3)                  # 30% null
created_at: datetime @auto @immutable                # system-generated
total: currency.usd @derived(sum(items.subtotal))    # computed field
```

### Relationships

```yaml
entities:
  User:
    id: uuid @primary
    orders: [Order] @count(0..50)           # has-many

  Order:
    id: uuid @primary
    customer: ->User                         # belongs-to (required)
    reviewer: ->User?                        # belongs-to (optional)
    parent: ->self?                          # self-referential

  Post:
    tags: <->Tag @count(1..5)               # many-to-many
    commentable: ->Post | ->Photo | ->Video  # polymorphic
```

### Full Example

```yaml
domain: project_management
version: 0.1.0
seed: 42
locale: en-US

volume:
  Organization: 5
  User: 50
  Project: 20
  Task: 200

enums:
  Priority: [critical, high, medium, low]
  TaskStatus: [backlog, todo, in_progress, review, done, cancelled]

entities:
  Organization:
    id: uuid @primary
    name: company.name @unique
    plan: enum(free, team, business, enterprise) @dist(40, 30, 20, 10)

  User:
    id: uuid @primary
    org: ->Organization
    name: person.full
    email: email @unique
    role: enum(admin, manager, member, viewer) @dist(5, 15, 70, 10)

  Project:
    id: uuid @primary
    org: ->Organization
    name: string @len(3..60)
    lead: ->User
    status: enum(planning, active, paused, completed, archived) @dist(10, 50, 10, 20, 10)

  Task:
    id: uuid @primary
    project: ->Project
    title: string @len(5..120)
    status: TaskStatus @dist(10, 15, 20, 15, 35, 5)
    priority: Priority @dist(5, 15, 50, 30)
    assignee: ->User? @null_rate(0.15)

rules:
  - Task.assignee.org == Task.project.org @strict
```

## CLI Reference

### generate

```
datjit generate <schema.yaml> [OPTIONS]

Options:
  -o, --output <path>           Output file (default: stdout)
  -f, --format <fmt>            json | csv | ndjson | yaml | sql (default: json)
      --seed <N>                Override schema seed
      --locale <bcp47>          Override locale
      --volume <Entity=N,...>   Override volume per entity
      --entity <name>           Generate only this entity (+ dependencies)
      --sql-dialect <dialect>   postgres | mysql | sqlite (default: postgres)
      --pretty                  Pretty-print JSON/YAML
      --dry-run                 Show generation plan without generating
```

### validate

```
datjit validate <schema.yaml>
```

Checks schema for parse and validation errors. Exits 0 if valid, 1 with error messages if not.

### inspect

```
datjit inspect <schema.yaml> [--infer-tools]
```

Prints parsed schema summary: entities, fields, dependency graph, volume plan, enums.
With `--infer-tools`, also prints the auto-generated CRUD tool surface per entity.

### corpus

```
datjit corpus list      # Show known data sources
datjit corpus info      # Show installed corpus data
datjit corpus update    # Download/refresh corpus sources
```

## Output Formats

**JSON** (default) — single object with entity arrays:
```json
{
  "User": [
    {"id": "abc-123", "name": "Sofia Patel", "email": "user3506@test.org", "age": 18, "active": false},
    ...
  ]
}
```

**CSV** — one header + data rows per entity:
```
id,name,email,age,active
abc-123,Sofia Patel,user3506@test.org,18,false
```

**SQL** — CREATE TABLE + batched INSERT statements:
```sql
CREATE TABLE "User" (
  "id" UUID,
  "name" TEXT,
  "email" TEXT,
  "age" BIGINT,
  "active" BOOLEAN
);

INSERT INTO "User" ("id", "name", "email", "age", "active") VALUES
  ('abc-123', 'Sofia Patel', 'user3506@test.org', 18, FALSE),
  ...
```

**NDJSON** — one JSON object per line.

**YAML** — single YAML document with all entities.

## Architecture

Hexagonal architecture with 6 Cargo workspace crates:

```
datjit-core        Domain types, port traits, errors (zero infra deps)
datjit-parser      YAML + embedded DSL parsing (recursive descent)
datjit-generator   Data generation engine (topological sort, distributions, coherence)
datjit-output      Format writers (JSON, CSV, SQL, YAML, NDJSON)
datjit-corpus      Corpus data management (embedded + downloadable)
datjit-cli         CLI entry point (clap)
```

Dependencies flow inward: adapters depend on core, core depends on nothing internal.

## Semantic Types

60+ built-in semantic types organized by namespace:

| Namespace | Types |
|-----------|-------|
| `person` | `full`, `first`, `last`, `prefix`, `username`, `gender`, `dob`, `age`, `bio`, `avatar` |
| `email`, `phone`, `url` | top-level contact types, `phone.mobile`, `url.image` |
| `address` | `full`, `street`, `city`, `state`, `zip`, `country` |
| `geo` | `lat`, `lng` |
| `currency` | `usd`, `eur`, `currency(CODE)` |
| `text` | `word`, `sentence`, `paragraph`, `slug`, `markdown` |
| `product` | `title`, `description`, `sku` |
| `company` | `name`, `industry`, `catch_phrase` |
| `job` | `title`, `department` |
| `color` | `hex`, `rgb`, `name` |
| `file` | `name`, `extension`, `mime` |
| `hash` | `md5`, `sha256` |

## Distributions

Control the statistical shape of generated data:

```yaml
age: int @range(18..95) @dist(normal, mu=35, sigma=12)
price: float @range(1..5000) @dist(lognormal, mu=3.5, sigma=1.2)
wait_time: float @dist(exponential, lambda=0.5)
popularity: int @dist(zipf, s=1.5)
tier: enum(free, pro, enterprise) @dist(70, 25, 5)
```

Supported: `uniform`, `normal`, `lognormal`, `exponential`, `geometric`, `zipf`, `bimodal`, `weighted`.

## Development

```bash
cargo build              # Build all crates
cargo test               # Run all 193 tests
cargo run -- --help      # Run CLI
```

## Specification

The full DDL language specification is in [docs/datjit-spec.md](docs/datjit-spec.md).
Corpus data sources are documented in [docs/datjit-sources.md](docs/datjit-sources.md).

## License

MIT
