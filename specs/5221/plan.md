# Issue #5221 Plan

- Issue: #5221
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Add a dedicated R43 decision-contract section to `docs/planning/kamn-data-layer-prd.docx.md` with stable marker keys.
2. Add a new docs-contract test file in `crates/kamn-core/tests/` that asserts marker presence and valid follow-up issue reference syntax.
3. Run targeted docs-contract test command as RED/GREEN evidence and regression verification.
4. Update issue/process markers and close with shell-surface actuals.

## Risks and Mitigations
- Risk: marker naming drift between docs and tests.
  - Mitigation: use a small, explicit marker vocabulary and keep test assertions centralized.
- Risk: brittle prose coupling.
  - Mitigation: assert only stable marker keys/values, not narrative wording.

## Interfaces / Contracts
- Marker schema uses explicit keys:
  - `data_layer_m11_operator_readiness_standalone_status`
  - `data_layer_prd_conformance_standalone_status`
  - `data_layer_standalone_reason_taxonomy_version`
  - `typed_did_migration_backlog_issue_ids`
