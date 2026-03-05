# Spec: Issue #6379 - M0-M11 boundary map + initial M11 extraction slice

## Objective

De-risk `kamn-core` monolith decomposition by shipping a concrete M0-M11 extraction map and landing one behavior-preserving extraction slice: moving M11 hardening matrix ownership into `kamn-data-layer` with compatibility shims in `kamn-core`.

## Inputs/Outputs

- Inputs:
  - `kamn-core` data-layer modules `data_layer_m0..data_layer_m11_*`
  - `kamn-data-layer` crate (currently hashing-only extraction)
  - existing runtime wiring doc: `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - explicit M0-M11 extraction ownership map with sequence, compatibility strategy, and contract-test protection points
  - extracted M11 hardening implementation in `kamn-data-layer`
  - compatibility-preserving re-export shim in `kamn-core`
  - pre/post telemetry for extracted slice test lane

## Boundaries/Non-goals

- In scope:
  - map all M0-M11 boundaries with target crate ownership
  - initial extraction slice for M11 hardening matrix contracts only
  - preserve existing `kamn-core` public API paths via shim/re-export compatibility
- Out of scope:
  - wholesale migration of all M0-M11 modules in one PR
  - API-breaking import-path changes for downstream users
  - runtime behavior redesign

## Failure modes

- FM-1: extraction map is incomplete or lacks explicit ownership by module milestone.
- FM-2: extracted M11 slice breaks existing `kamn-core` imports/behavior.
- FM-3: extraction ships without pre/post telemetry evidence for the slice.

## Acceptance criteria (testable booleans)

- [x] AC-1: M0-M11 extraction boundary map is documented with explicit ownership targets.
- [x] AC-2: extraction sequence includes compatibility strategy and contract-test protection points.
- [x] AC-3: M11 hardening initial extraction slice lands with no behavior regressions.
- [x] AC-4: pre/post build/test telemetry is captured for the extracted slice.

## Files to touch

- `specs/6379-m0-m11-boundary-map-and-m11-hardening-extraction.md`
- `docs/architecture/data-layer-runtime-wiring.md`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs` (new)
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m11_hardening_readiness.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m11_hardening_readiness_integration.rs` (new)
- `crates/kamn-core/src/data_layer_m11_hardening_readiness.rs`

## Error semantics

- Compatibility shim must remain fail-closed and preserve typed error taxonomy.
- Contract tests must fail on extraction-map marker drift.
- Telemetry capture commands must be deterministic and comparable pre/post.

## Test plan

- RED:
  - add docs contract requiring M0-M11 extraction map/sequence/compatibility markers.
  - add extraction ownership guard expecting `kamn-data-layer` to expose M11 hardening surface.
- GREEN:
  - update architecture doc with extraction map + protection points.
  - move M11 hardening implementation to `kamn-data-layer`.
  - keep `kamn-core` module shim + public re-export compatibility.
- REFACTOR:
  - consolidate duplicate exports/imports and keep shim minimal.
- INTEGRATION:
  - run `kamn-core` M11 hardening + closure suites via public API.
  - run `kamn-data-layer` M11 hardening integration tests.
  - capture pre/post telemetry for `kamn-core` M11 hardening test lane.

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs` (pass)
- 2026-03-05: `cargo test -p kamn-data-layer --test data_layer_m11_hardening_readiness_integration` (pass)
- 2026-03-05: `cargo test -p kamn-core --test data_layer_m11_hardening_readiness` (pass)
- 2026-03-05: `cargo test -p kamn-core --test data_layer_m11_closure_evidence` (pass)
- 2026-03-05 telemetry:
  - pre extraction lane timing: `m11_core_test_pre_seconds=0.33`
    - command: `/usr/bin/time -f 'm11_core_test_pre_seconds=%e' -o /tmp/m11_core_pre.time cargo test -p kamn-core --test data_layer_m11_hardening_readiness`
  - post extraction lane timing: `m11_core_test_post_seconds=0.12`
    - command: `/usr/bin/time -f 'm11_core_test_post_seconds=%e' -o /tmp/m11_core_post.time cargo test -p kamn-core --test data_layer_m11_hardening_readiness`

## Deviations

- None.
