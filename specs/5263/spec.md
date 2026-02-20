# Issue #5263 Spec

- Title: Task: implement Phase-2 envelope+blind-index operational pipeline and persist blind indexes in adapter inserts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
The runtime lacks a Phase-2 operational pipeline that composes encryption artifacts with blind-index generation, and adapter inserts currently hardcode `blind_indexes` as `{}` instead of persisting derived tokens.

## Scope
In:
- Add a new `kamn-core` Phase-2 operational pipeline module that composes `direct_message_crypto`, `DataLayerM0EnvelopeRecord::derive`, and `data_layer_m3_compute_blind_index`.
- Emit deterministic pipeline artifacts with encrypted M0 record and blind-index token map.
- Extend PostgreSQL execution adapter insert path to accept persisted blind-index JSON payloads.
- Add tests for deterministic output, fail-closed invalid inputs, and insert+search integration.

Out:
- New/changed cryptography dependencies.
- pgvector/AGE/Timescale execution lanes.
- Kolme anchoring/runtime batching changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 420
- shell_to_rust_ratio_delta_estimate: -0.0022
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Phase-2 pipeline derives deterministic encrypted M0 record and blind-index tokens for valid inputs.
- AC-2: Pipeline fails closed for malformed key refs, missing recipient bindings, and invalid blind-index inputs.
- AC-3: Adapter insert path persists caller-provided blind-index token map and search retrieves rows by derived token.
- AC-4: Targeted tests and quality gates pass (`fmt`, strict `clippy`, adapter/bridge/migration/public-api suites).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid envelope + key refs + blind-index fields | deterministic artifact with non-empty ciphertext, wrapped keys, and token map |
| C-02 | AC-2 | Regression | malformed sender/recipient key refs, missing recipient binding, empty blind-index key material | fail-closed structured errors |
| C-03 | AC-3 | Integration | insert record with derived blind indexes + blind-index search request | inserted row retrievable by derived token |
| C-04 | AC-4 | Verification | fmt/clippy + target suites | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_phase2_crypto_blind_index_pipeline`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- `cargo test -p kamn-core --test public_api_surface_policy`

## Success Metrics
- Runtime now has an operational Phase-2 pipeline output usable by persistence layers.
- Insert path no longer discards blind-index tokens before storage.
