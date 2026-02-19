# Issue #5193 Plan

- Issue: #5193
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. Build a new core shared harness (`service_api_docs_contract_harness.rs`) that absorbs marker assertions from:
   - `service_api_contract_docs.rs`
   - `service_api_lifecycle_contract_docs.rs`
2. Add a core migration-contract suite to verify superseded files are retired and migrated case IDs exist in the new harness.
3. Refactor sdk docs suite (`rust_sdk_alpha_docs.rs`) to a case-matrix harness structure while preserving marker parity.
4. Add a template-guidance contract test and update `.github/ISSUE_TEMPLATE/subtask.md` with a docs-contract migration checklist.
5. Run targeted core+sdk docs suites plus template guidance contract and strict lint/format gates.

## Affected Modules / Files
- New:
  - `crates/kamn-core/tests/service_api_docs_contract_harness.rs`
  - `crates/kamn-core/tests/service_api_docs_harness_migration_contract.rs`
  - `crates/kamn-core/tests/docs_contract_template_guidance_contract.rs`
- Updated:
  - `crates/kamn-sdk/tests/rust_sdk_alpha_docs.rs`
  - `docs/service/api-contract.md`
  - `.github/ISSUE_TEMPLATE/subtask.md`
- Deleted:
  - `crates/kamn-core/tests/service_api_contract_docs.rs`
  - `crates/kamn-core/tests/service_api_lifecycle_contract_docs.rs`

## Risks and Mitigations
- Risk: marker drift while consolidating service API docs assertions.
  - Mitigation: map each legacy assertion to explicit matrix case markers and keep stable inventory-size regression checks.
- Risk: ratio/non-regression policies fail if test file count regresses.
  - Mitigation: pair suite removals with explicit new harness/migration contract files in same slice.
- Risk: future issues skip matrix migration pattern.
  - Mitigation: template checklist marker plus contract test for guidance marker presence.

## Interfaces / Contracts
- Shared harness contract requirements:
  - deterministic case IDs
  - explicit document labels
  - non-empty marker lists
- Template guidance contract marker:
  - `docs_contract_matrix_migration_checklist_status=required-when-applicable`
