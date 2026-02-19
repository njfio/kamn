# Issue #5192 Spec

- Title: Subtask: migrate remaining kamn-node docs-contract suites to shared harness matrix
- Status: Accepted
- Priority: P2
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
`kamn-node` still carries one-file-per-doc contract suites for several pure markdown contract checks. This keeps test file count high and duplicates assertion structure already solved by the data-driven case-matrix harness pattern.

## Scope
In:
- Consolidate the following pure docs-contract suites into the shared `node_runtime_cli_docs` matrix harness:
  - `kolme_runtime_commit_docs.rs`
  - `runtime_processor_ha_docs.rs`
  - `r42_api_shell_surface_audit_docs.rs`
  - `signer_migration_parity_docs_contract.rs`
- Preserve marker parity and deterministic case IDs.
- Remove superseded files after parity migration.
- Update affected docs command markers where old suite names are retired.

Out:
- Migration of mixed docs+source contract suites (`observability_contracts_docs.rs`, signer/source boundary suites, etc.).
- CI workflow script changes.

## Acceptance Criteria
- AC-1: The four migration inventory suites are represented in a shared data-driven harness matrix with deterministic case IDs.
- AC-2: The superseded inventory files are removed and `kamn-node` test file count decreases.
- AC-3: Targeted docs-contract lanes remain green after migration and command-marker updates.

## Migration Inventory
1. `crates/kamn-node/tests/kolme_runtime_commit_docs.rs`
2. `crates/kamn-node/tests/runtime_processor_ha_docs.rs`
3. `crates/kamn-node/tests/r42_api_shell_surface_audit_docs.rs`
4. `crates/kamn-node/tests/signer_migration_parity_docs_contract.rs`

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Shared harness source | New matrix contains inventory docs and required marker sets with stable case IDs |
| C-02 | AC-2 | Regression | Git-tracked `kamn-node/tests` files | All four inventory files are removed; net test file count decreases |
| C-03 | AC-3 | Functional | Targeted docs-contract test runs | Shared harness and migration contract suites pass with updated command markers |

## Test Mapping
- C-01/C-03 -> `cargo test -p kamn-node --test node_runtime_cli_docs -- --nocapture`
- C-02/C-03 -> `cargo test -p kamn-node --test doc_contract_harness_migration_contract`
- C-03 -> `cargo test -p kamn-node --test architecture_navigation_docs`

## Success Metrics
- `kamn-node/tests` file count decreases by at least 4.
- No marker parity regressions in migrated docs sections.
- Shell LOC delta remains `0`.
