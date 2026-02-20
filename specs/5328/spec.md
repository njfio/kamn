# Issue #5328 Spec

- Title: Decompose `data_layer_postgres_execution_adapter` into focused runtime submodules
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
`crates/kamn-core/src/data_layer_postgres_execution_adapter.rs` reached 1,114 LOC and mixes adapter core logic with migration parsing, row codec, validation helpers, and error taxonomy. This reduces maintainability and readability.

## Acceptance Criteria
- AC-1: Root `data_layer_postgres_execution_adapter.rs` is reduced below 900 LOC by extracting helper concerns into child modules.
- AC-2: Public adapter API and behavior remain stable for existing tests/callers.
- AC-3: Existing adapter conformance/integration suites pass without behavioral drift.
- AC-4: `cargo clippy -p kamn-core -- -D warnings` remains clean.

## Scope
In scope:
- Extract helper concerns into `data_layer_postgres_execution_adapter/{error,migration,codec,validation}.rs`.
- Preserve public symbols and reason-code semantics.

Out of scope:
- Runtime behavior changes.
- Dependency changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Structural | LOC scan on root adapter file | root LOC < 900 |
| C-02 | AC-2 | Integration | existing adapter/public API tests | no API/behavior drift |
| C-03 | AC-3 | Conformance | `cargo test -p kamn-core --test data_layer_postgres_execution_adapter` | suite passes unchanged |
| C-04 | AC-4 | Quality | `cargo clippy -p kamn-core -- -D warnings` | zero warnings |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test data_layer_phase2_crypto_blind_index_pipeline`
- `cargo test -p kamn-core --test public_api_surface_policy`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Root adapter file exits >1K band and stays below 900 LOC.
- Adapter and related integration suites remain green.
