# Spec: Issue #6217 - Fuzz Target for Kolme Flat-JSON Parser Surfaces

- Status: Implemented
- Priority: P2
- Parent: #6183
- Milestone: R59 Swarm Gap Closure

## Problem Statement

R59 identified parser-risk surfaces in `kamn-kolme` (`split_unquoted` and
`parse_flat_json_value_fields`) that should have dedicated fuzz coverage. Current fuzz targets do
not explicitly model this surface as a first-class corpus target.

## Scope

In scope:
- Add dedicated cargo-fuzz target for Kolme flat-json/provider-response parsing surfaces.
- Add deterministic seed corpus for the new target.
- Extend replay metadata and contract tests to require the new target/corpus markers.

Out of scope:
- Full parser rewrite.
- Always-on deep fuzz execution in CI fast gate.

## Acceptance Criteria

### AC-1 Dedicated Target Exists
Given the fuzz package,
When targets are enumerated,
Then a dedicated Kolme flat-json parser target is present.

### AC-2 Seed Corpus and Metadata
Given deterministic replay metadata,
When required targets/corpora are validated,
Then the new target and seed files are listed.

### AC-3 Regression Contracts
Given contract tests,
When parser fuzz governance tests run,
Then they fail closed if the target/corpus markers are removed.

## Conformance Cases

- C-01 (AC-1, Functional): `fuzz/Cargo.toml` includes `kolme_flat_json_parser` target.
- C-02 (AC-2, Regression): replay metadata includes `kolme_flat_json_parser` with seed files.
- C-03 (AC-3, Regression): `cargo_fuzz_target_contract` asserts target + metadata markers.

## Success Metrics

- `cargo test -p kamn-core --test cargo_fuzz_target_contract` passes.
- New target compiles via fuzz workspace metadata checks.
