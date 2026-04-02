# datjit — synthetic data generation from DDL schemas

default:
    @just --list

# Build all crates
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Check compilation without building
check:
    cargo check --all-targets

# Run clippy lints
lint:
    cargo clippy --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Generate data from a schema (JSON to stdout)
generate schema *args:
    cargo run --quiet -- generate {{schema}} {{args}} 2>/dev/null

# Generate with a specific format
generate-format schema format *args:
    cargo run --quiet -- generate {{schema}} -f {{format}} {{args}} 2>/dev/null

# Validate a DDL schema
validate schema:
    cargo run --quiet -- validate {{schema}}

# Inspect a DDL schema
inspect schema *args:
    cargo run --quiet -- inspect {{schema}} {{args}}

# Inspect with tool inference
inspect-tools schema:
    cargo run --quiet -- inspect {{schema}} --infer-tools

# Show generation plan without generating
dry-run schema *args:
    cargo run --quiet -- generate {{schema}} --dry-run {{args}}

# List available corpus sources
corpus-list:
    cargo run --quiet -- corpus list

# Show corpus status
corpus-info:
    cargo run --quiet -- corpus info

# Update corpus data from remote sources
corpus-update:
    cargo run --quiet -- corpus update

# Run the minimal example
example-minimal:
    cargo run --quiet -- generate tests/fixtures/minimal.yaml --seed 42 2>/dev/null

# Run the project management example
example-pm:
    cargo run --quiet -- generate tests/fixtures/project_management.yaml --seed 42 2>/dev/null

# Run project management example as CSV
example-pm-csv:
    cargo run --quiet -- generate tests/fixtures/project_management.yaml --seed 42 -f csv 2>/dev/null

# Run project management example as SQL
example-pm-sql:
    cargo run --quiet -- generate tests/fixtures/project_management.yaml --seed 42 -f sql 2>/dev/null

# Count lines of Rust source code
loc:
    @find crates -name '*.rs' | xargs wc -l | tail -1

# Count tests
test-count:
    @cargo test 2>&1 | grep "^test result" | awk -F'[;,]' '{for(i=1;i<=NF;i++) if($i~/passed/) {gsub(/[^0-9]/,"",$i); sum+=$i}} END{print sum " tests"}'

# Clean build artifacts
clean:
    cargo clean

# Full CI check: fmt, lint, test
ci: fmt-check lint test
