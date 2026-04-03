# Plan: Add Missing JSON Schema Constraints to datjit DDL

## Context

JSON Schema provides validation constraints that cannot currently be expressed in datjit's DDL. This plan adds the missing constraints that are meaningful for synthetic data generation, ensuring datjit can express anything JSON Schema can.

## Gap Analysis

| JSON Schema Constraint | Current datjit Support | Action |
|---|---|---|
| exclusiveMinimum/Maximum | `@range` is inclusive only | Add exclusive bound syntax |
| multipleOf | Not supported | Add `@multiple_of(n)` |
| uniqueItems | Not supported for lists | Add `@unique_items` |
| const | `@values(x)` works but undocumented | Document |
| minItems/maxItems | `@len` on lists pads with Null | Fix list generation to use `@len` directly |
| dependentRequired | Not supported | Add `@dependent_required(fields...)` |
| deprecated | Not supported | Add `@deprecated` |
| writeOnly | Not supported | Add `@write_only` |
| examples | Not supported | Add `@examples(v1, v2)` |
| not/allOf/oneOf | Not meaningful for data gen | Skip |
| patternProperties/additionalProperties | Not meaningful for data gen | Skip |
| minProperties/maxProperties | Marginal value | Skip |
| contains/minContains/maxContains | Complex, marginal value | Skip |

## Implementation

### Phase 1: Exclusive Bounds

**Syntax:** `@range(0<..100)` (exclusive lower), `@range(0..<100)` (exclusive upper), `@range(0<..<100)` (both). Also `@emin(n)` and `@emax(n)` as shorthand decorators.

**Files:**

- `crates/datjit-core/src/model/decorator.rs:14` — Change `Range(RangeValue, RangeValue)` to struct variant:
  ```rust
  Range { lo: RangeValue, hi: RangeValue, lo_exclusive: bool, hi_exclusive: bool },
  ```
  Change `Min(RangeValue)` to `Min { value: RangeValue, exclusive: bool }` and same for `Max`.

- `crates/datjit-parser/src/decorator_parser.rs` — In `parse_range`, detect `<..` / `..<` / `<..<` tokens. Add `"emin"` and `"emax"` match arms.

- `crates/datjit-generator/src/field_gen.rs:477-502` — `apply_range_constraint`: for exclusive bounds, use `lo + 1` (int) or `lo + f64::EPSILON` (float) and `hi - 1` / `hi - f64::EPSILON`.

- `crates/datjit-generator/src/field_gen.rs:152-169` — `extract_range_f64`: pass exclusivity info through.

- `crates/datjit-generator/src/decorator_apply.rs:38-70` — `apply_min`/`apply_max`: adjust for exclusive flag.

- Update all `Decorator::Range(..)` pattern matches across the codebase (grep shows: `field_gen.rs`, `decorator_apply.rs`, `constraint.rs`).

### Phase 2: multipleOf

**Syntax:** `@multiple_of(0.05)`, `@multiple_of(5)`

**Files:**

- `crates/datjit-core/src/model/decorator.rs` — Add `MultipleOf(f64)` variant.
- `crates/datjit-parser/src/decorator_parser.rs` — Add `"multiple_of"` match arm, parse single float arg.
- `crates/datjit-generator/src/decorator_apply.rs` — Add to `apply_single_decorator`:
  ```rust
  Decorator::MultipleOf(step) => {
      match value {
          Value::Int(n) => Ok(Value::Int((n as f64 / step).round() as i64 * step as i64)),
          Value::Float(n) => Ok(Value::Float((n / step).round() * step)),
          _ => Ok(value),
      }
  }
  ```

### Phase 3: uniqueItems

**Syntax:** `@unique_items`

**Files:**

- `crates/datjit-core/src/model/decorator.rs` — Add `UniqueItems` variant.
- `crates/datjit-parser/src/decorator_parser.rs` — Add `"unique_items"` to no-args match.
- `crates/datjit-generator/src/field_gen.rs:113-119` — In `CompoundType::List` branch, if `UniqueItems` in decorators, use a `HashSet` to deduplicate during generation with retry limit (100 attempts).

### Phase 4: Fix @len on lists (minItems/maxItems)

**File:** `crates/datjit-generator/src/field_gen.rs:113-119`

Currently list generation uses `rng.gen_range(0..5)` ignoring `@len`. Fix: extract `Len(lo, hi)` from decorators and use that as the count range. Remove Null-padding in `decorator_apply.rs:96-104`.

### Phase 5: const (documentation only)

**File:** `docs/datjit-spec.md`

Document that `@values(x)` with a single value is the equivalent of JSON Schema `const`:
```yaml
# JSON Schema: { "const": "USD" }
currency_code: string @values(USD)
```

### Phase 6: dependentRequired

**Syntax:** `@dependent_required(shipping_date, tracking_number)`

**Files:**

- `crates/datjit-core/src/model/decorator.rs` — Add `DependentRequired(Vec<String>)` variant.
- `crates/datjit-parser/src/decorator_parser.rs` — Parse comma-separated field names.
- `crates/datjit-generator/src/engine.rs` — After generating all fields for a row, check `DependentRequired` decorators. If the field is non-null, force regeneration of dependent fields to non-null.

### Phase 7: Metadata decorators (deprecated, writeOnly, examples)

**Syntax:** `@deprecated`, `@write_only`, `@examples("foo", "bar")`

**Files:**

- `crates/datjit-core/src/model/decorator.rs` — Add `Deprecated`, `WriteOnly`, `Examples(Vec<String>)` variants.
- `crates/datjit-parser/src/decorator_parser.rs` — Add match arms. `deprecated`/`write_only` are no-args. `examples` parses comma-separated strings.
- No generator changes — these are metadata-only (pass through via the `_ => Ok(value)` default).
- `crates/datjit-core/src/model/tool_inference.rs` — `@write_only` fields should appear in create/update inputs but not read/list outputs. `@deprecated` fields could get a deprecation annotation.

### Phase 8: Spec and tests

- `docs/datjit-spec.md` — Document all new decorators with examples.
- Add test fixture `tests/fixtures/json_schema_constraints.yaml` exercising exclusive ranges, multipleOf, uniqueItems, dependentRequired, const via @values, metadata decorators.
- Add integration test in `crates/datjit-cli/tests/integration_test.rs`.

## Verification

```bash
cargo test                                    # all tests pass
cargo clippy --all-targets -- -D warnings     # no warnings
cargo fmt --all -- --check                    # formatted
cargo run -- generate tests/fixtures/json_schema_constraints.yaml --seed 42  # new fixture works
cargo run -- inspect tests/fixtures/json_schema_constraints.yaml             # decorators visible
```
