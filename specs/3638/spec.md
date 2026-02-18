# Issue #3638 Spec

- Title: `Task: deliver signer parity harness and migration completion`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer extraction reduced monolith size, but migration completion still needs explicit parity matrix and guardrails that fail closed on drift.

## Scope
In:
- Add signer parity matrix and deterministic drift-guard coverage for profile/key-source and reason-code contracts.
- Keep signer ownership boundaries explicit and regression-guarded.
- Update signer migration docs/runbooks with parity guard entrypoints.

Out:
- New signer algorithms/providers.
- Runtime protocol changes.

## Acceptance Criteria
- AC-1: Given signer profile/key-source migration contracts, when parity tests run, then primary/secondary/env-local/managed-external behavior contracts are covered.
- AC-2: Given signer extraction artifacts, when drift checks run, then behavior/reason-marker drift fails closed.
- AC-3: Given scoped signer verification, when regression suites run, then signer behavior remains parity-stable.
- AC-4: Given architecture docs, when docs parity tests run, then signer migration matrix and guard commands remain documented.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `cargo test -p kamn-node main_tests::signer_tests::functional_signer_migration_profile_key_source_parity_matrix -- --exact --nocapture` | profile/key-source parity matrix remains covered and green |
| C-02 | AC-2 | Regression/Conformance | `cargo test -p kamn-node --test signer_migration_parity_docs_contract -- --nocapture` | docs/source parity guard fails closed on migration marker drift |
| C-03 | AC-3 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` + `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer behavior remains parity-stable |
| C-04 | AC-4 | Docs/Conformance | `cargo test -p kamn-node --test signer_migration_parity_docs_contract docs_signer_lifecycle_declares_migration_parity_matrix -- --exact --nocapture` | signer lifecycle docs include migration parity matrix and guard commands |

## Test Mapping
- C-01: `crates/kamn-node/src/main_tests/signer_tests.rs`
- C-02/C-04: `crates/kamn-node/tests/signer_migration_parity_docs_contract.rs`
- C-03: existing signer runtime and integration suites

## Success Metrics
- Signer migration parity matrix is explicitly tested and documented.
- Drift guard catches missing/renamed migration markers.
- Scoped signer regression suites remain green.
