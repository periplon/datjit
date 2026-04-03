# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is datjit

Synthetic data generator that takes declarative YAML schemas (using a DDL — Data Domain Language) and produces realistic test data in JSON, CSV, SQL, YAML, or NDJSON. Fields combine a type (primitive, semantic, enum, reference, compound) with decorators that control generation behavior.

## Commands

```bash
# Build
cargo build                          # all crates
cargo build --release                # release binary

# Test
cargo test                           # all tests
cargo test -p datjit-generator       # single crate
cargo test test_basic_generation     # single test by name

# Lint & format
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo fmt --all -- --check

# Full CI check (fmt + lint + test)
just ci

# Run CLI during development
cargo run -- generate tests/fixtures/minimal.yaml --seed 42
cargo run -- validate tests/fixtures/project_management.yaml
cargo run -- inspect tests/fixtures/project_management.yaml --infer-tools
```

A `justfile` provides shortcuts: `just test`, `just lint`, `just fmt`, etc.

## Architecture

Hexagonal architecture — dependencies flow inward, core has zero infra deps.

```
datjit-core       Domain model + port traits (DdlParser, DataGenerator, OutputWriter, CorpusProvider)
datjit-parser     YAML parsing + embedded DDL type/decorator parsing (recursive descent)
datjit-generator  Generation engine: topological sort → coherence groups → field gen → decorators → derived → rules
datjit-output     Format writers implementing OutputWriter (JSON, CSV, SQL, YAML, NDJSON)
datjit-corpus     Embedded + downloadable corpus data for semantic types
datjit-cli        Clap CLI entry point with subcommands: generate, validate, inspect, corpus
```

### Generation pipeline (datjit-generator/src/engine.rs)

For each entity in topological order:
1. Generate coherence groups (correlated fields)
2. Generate non-derived fields (primary keys, auto fields, then regular fields with decorator application)
3. Enforce uniqueness constraints via retry loop
4. Evaluate `@derived` fields from expressions
5. Add `@timestamps` if applicable
6. Enforce rules with retry (up to 10 attempts per row)

### Core domain model (datjit-core)

- `DdlDocument` — top-level parsed schema (domain, entities, enums, rules, volume specs)
- `Entity` / `Field` — entity definitions with typed fields and decorators
- `TypeExpr` — type system: `Primitive`, `Semantic`, `Enum`, `Reference`, `Compound`, `Named`
- `Decorator` — generation modifiers: `@primary`, `@unique`, `@range`, `@dist`, `@pattern`, `@derived`, etc.
- Port traits in `ports/`: `DdlParser`, `DataGenerator`, `OutputWriter`, `CorpusProvider`

### Error handling

Single unified error type `DatjitError` (datjit-core/src/error.rs) used across all crates. All port traits return `Result<T, DatjitError>`. Variants carry context — e.g., `Parse { location, message }`, `UniquenessExhausted { entity, field, attempts }`.

### Parser internals (datjit-parser)

Type parser (`type_parser.rs`) is recursive descent with this precedence (top = loosest binding):
1. Union (`T1 | T2`) → Nullable (`T?`) → Compound (`[T]`, `{K:V}`) → Reference (`->Entity`) → Enum → Parameterized → Bare primitive/semantic/named

Decorator parser (`decorator_parser.rs`) uses a stateful tokenizer that tracks parenthesis depth to split `type @dec1(x,y) @dec2` cleanly.

### Corpus system (datjit-corpus)

Two-tier fallback: embedded hardcoded arrays (for zero-config) → external JSON files from `~/.datjit/corpus/`. Corpus entries use `{name, weight}` structure for weighted sampling. `CorpusRegistry` tries external files first, falls back to embedded. Updater downloads from US Census, GeoNames, O*NET, IANA, GitHub raw sources.

### Dependency resolution

Topological sort via Kahn's algorithm (`plan.rs`). Self-references don't count as dependencies. Ties broken by document definition order for determinism.

### Key patterns

- `IndexMap` used throughout for deterministic field/entity ordering
- `Value` enum (`value.rs`) is the universal runtime value type
- Seed-based deterministic generation via `rand_chacha`
- No feature flags — builds are monolithic across all crates

## Testing

Integration tests live in `crates/datjit-cli/tests/`. They chain YamlParser → GenerationEngine → output through the port traits, simulating the real CLI pipeline. Fixtures in `tests/fixtures/*.yaml` cover every DDL feature (primitives, semantic types, enums, decorators, references, coherence groups, rules, derived fields, compound types, named types). Tests strip non-deterministic UUID `id` fields to verify seeded consistency.

```bash
cargo test                           # all tests
cargo test -p datjit-generator       # single crate
cargo test test_basic_generation     # single test by name
```

## DDL spec

Full language specification is in `docs/datjit-spec.md`. Corpus sources documented in `docs/datjit-sources.md`.
