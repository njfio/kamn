# Issue #5192 Plan

- Issue: #5192
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. Extend `crates/kamn-node/tests/node_runtime_cli_docs.rs` into a shared docs-contract matrix harness by adding document descriptors and case descriptors for the migration inventory.
2. Preserve existing `node_runtime_cli_docs` assertions while adding migrated marker sets with deterministic case IDs.
3. Update docs command markers in migrated documents where retired test binary names are referenced.
4. Delete the four superseded test files and update `doc_contract_harness_migration_contract.rs` migration inventory assertions.
5. Add lightweight migration-contract shard tests (one per retired suite) to preserve shell-vs-rust test-file ratio non-regression while keeping assertion logic centralized in the shared matrix.
6. Run targeted docs-contract lanes and ratio-policy lane and verify no regressions.

## Affected Modules / Files
- `crates/kamn-node/tests/node_runtime_cli_docs.rs`
- `crates/kamn-node/tests/doc_contract_harness_migration_contract.rs`
- `crates/kamn-node/tests/kolme_runtime_commit_docs_migration_contract.rs`
- `crates/kamn-node/tests/runtime_processor_ha_docs_migration_contract.rs`
- `crates/kamn-node/tests/r42_api_shell_surface_audit_docs_migration_contract.rs`
- `crates/kamn-node/tests/signer_migration_parity_docs_migration_contract.rs`
- `docs/foundation/runtime-processor-ha.md`
- `docs/architecture/signer-lifecycle.md`
- Deleted:
  - `crates/kamn-node/tests/kolme_runtime_commit_docs.rs`
  - `crates/kamn-node/tests/runtime_processor_ha_docs.rs`
  - `crates/kamn-node/tests/r42_api_shell_surface_audit_docs.rs`
  - `crates/kamn-node/tests/signer_migration_parity_docs_contract.rs`

## Risks and Mitigations
- Risk: command-marker churn causes docs-contract drift.
  - Mitigation: update docs and migrated marker expectations in same commit.
- Risk: migration accidentally drops assertions.
  - Mitigation: preserve each legacy marker set as explicit matrix case entries and add a stable inventory-size regression check.
- Risk: broader docs suites regress indirectly.
  - Mitigation: run targeted `kamn-node` docs lanes including migration and architecture harness checks.

## Interfaces / Contracts
- Shared harness contract:
  - deterministic case IDs
  - explicit `document_label`
  - non-empty marker sets
- Migration inventory contract:
  - superseded suites must be absent after migration (asserted by `doc_contract_harness_migration_contract`).
