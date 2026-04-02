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

### Key patterns

- `IndexMap` used throughout for deterministic field/entity ordering
- `Value` enum (`value.rs`) is the universal runtime value type
- Seed-based deterministic generation via `rand_chacha`
- Test fixtures in `tests/fixtures/*.yaml`

## DDL spec

Full language specification is in `docs/datjit-spec.md`. Corpus sources documented in `docs/datjit-sources.md`.
