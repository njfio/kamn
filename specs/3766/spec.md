# Issue #3766 Spec

- Title: `Subtask: add signer migration parity matrix and legacy-behavior diff guard`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer extraction must preserve legacy profile/key-source behavior and fail-closed reason taxonomy. A dedicated parity matrix and docs/source drift guard are required to prevent silent regression.

## Scope
In:
- Add signer migration parity matrix coverage for supported profile/key-source contracts.
- Add deterministic drift guard for signer migration docs markers.
- Update signer lifecycle docs with explicit parity matrix and guard command.

Out:
- New signing capabilities/providers.
- Performance/throughput redesign.

## Acceptance Criteria
- AC-1: Given signer profile/key-source contracts, when parity matrix tests run, then matrix behavior remains stable for supported combinations.
- AC-2: Given signer migration docs, when docs parity contracts run, then required matrix markers and guard commands remain present.
- AC-3: Given signer regression suites, when scoped tests run, then behavior and reason markers remain parity-stable.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `cargo test -p kamn-node main_tests::signer_tests::functional_signer_migration_profile_key_source_parity_matrix -- --exact --nocapture` | profile/key-source matrix contracts pass |
| C-02 | AC-2 | Docs/Conformance | `cargo test -p kamn-node --test signer_migration_parity_docs_contract -- --nocapture` | docs parity contract passes for migration matrix markers |
| C-03 | AC-3 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` + `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer behavior remains parity-stable |

## Test Mapping
- C-01: `crates/kamn-node/src/main_tests/signer_tests.rs`
- C-02: `crates/kamn-node/tests/signer_migration_parity_docs_contract.rs`
- C-03: existing signer suites

## Success Metrics
- Parity matrix coverage is explicit and deterministic.
- Docs/source drift guard fails closed on marker loss/rename.
- Signer suites remain green.
