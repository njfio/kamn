# Plan: Issue #6217 - Fuzz Target for Kolme Flat-JSON Parser Surfaces

## Approach

1. Add `kamn-kolme` dependency in `fuzz/Cargo.toml` and register new bin target
   `kolme_flat_json_parser`.
2. Implement target to exercise:
   - `parse_flat_json_value_fields`
   - `parse_provider_response_fields`
   These cover `split_unquoted` transitively.
3. Add deterministic seed corpus directory for the new target.
4. Extend replay metadata and `cargo_fuzz_target_contract` marker assertions.

## Affected Modules

- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/kolme_flat_json_parser.rs` (new)
- `fuzz/corpus/kolme_flat_json_parser/*` (new)
- `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`

## Risks and Mitigations

- Risk: metadata drift across fuzz targets.
  - Mitigation: extend existing contract tests to fail closed on marker removal.
- Risk: malformed input panics in parsing stack.
  - Mitigation: target only calls parser APIs and accepts both `Ok`/`Err` without assertions that
    could panic.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo test -p kamn-core --test cargo_fuzz_target_contract -- --nocapture`
