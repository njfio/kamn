# 6651 Increase Test Density In Utility Crates Wave 1

## Objective

Increase direct public-surface regression coverage in the first Wave 1 utility crates by adding high-signal tests for `kamn-types` DID boundary behaviors and `kamn-snapshot-journal` serialization round-trips, while explicitly recording why `kamn-bridges` is deferred from this wave.

## Inputs/Outputs

- Inputs:
  - Public DID boundary helpers and reexports in `crates/kamn-types/src/lib.rs`
  - Snapshot-journal append/parse/decode helpers in `crates/kamn-snapshot-journal/src/lib.rs`
  - Existing integration/contract tests in both crates
  - Issue inventory suggesting `kamn-types`, `kamn-snapshot-journal`, and `kamn-bridges`
- Outputs:
  - New public-surface regression tests in `kamn-types`
  - New append/parse round-trip tests in `kamn-snapshot-journal`
  - Explicit before/after test inventory evidence for the selected crates
  - Spec rationale recording why `kamn-bridges` is deferred from Wave 1

## Boundaries/Non-goals

- Do not redesign coverage policy across the repo
- Do not add mutation/fuzz coverage in this issue
- Do not add low-value tests that only restate implementation details without public contract value
- Do not expand this wave to `kamn-bridges` unless the existing direct coverage proves materially weaker than the source audit indicates

## Failure Modes

- Wave 1 selection is arbitrary and not justified by public-surface/test inventory evidence
- `kamn-types` still lacks direct tests for reexported DID boundary behaviors that downstream crates rely on
- `kamn-snapshot-journal` still lacks append/parse round-trip coverage for payload classes that matter at the public API boundary
- New tests duplicate existing `kamn-core` coverage instead of validating the utility-crate contract surface
- The final evidence does not state the before/after test counts for the selected crates

## Acceptance Criteria

- [x] Wave 1 selects 2 crates with explicit rationale
- [x] Missing public-surface tests are added for the selected crates
- [x] Serialization round-trip coverage is added where relevant
- [x] Regression tests cover known weak or previously untested paths
- [x] The selected crates have explicit before/after test inventory evidence

## Files To Touch

- `specs/6651-increase-test-density-in-utility-crates-wave-1.md`
- `crates/kamn-types/tests/did_boundary_regressions.rs`
- `crates/kamn-snapshot-journal/tests/snapshot_journal_roundtrip_integration.rs`

## Error Semantics

- `kamn-types` tests must assert typed DID boundary failures rather than string-only failures
- `kamn-snapshot-journal` tests must preserve fail-closed behavior for invalid parse/decode cases while proving successful round-trips where expected
- Test evidence should distinguish public utility-crate coverage from broader `kamn-core` transitive coverage

## Test Plan

- Run `cargo test -p kamn-types --test did_boundary_regressions -- --nocapture`
- Run `cargo test -p kamn-snapshot-journal --test snapshot_journal_roundtrip_integration -- --nocapture`
- Run `cargo test -p kamn-types -- --nocapture`
- Run `cargo test -p kamn-snapshot-journal -- --nocapture`

## Notes / Deviations

- Wave 1 intentionally selects `kamn-types` and `kamn-snapshot-journal`.
- `kamn-bridges` is deferred because it already has direct unit + integration coverage across normalization and settlement decision branches in both source and crate tests; that crate remains a follow-up candidate, but not the highest-leverage first wave after current repo inspection.
- Before Wave 1, inventory is:
  - `kamn-types`: 19 tests across 4 integration targets plus 5 unit tests in `src/lib.rs`
  - `kamn-snapshot-journal`: 14 tests across 3 integration targets plus 4 unit tests in `src/lib.rs`
