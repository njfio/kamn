# Spec — Issue #4198

- Title: Implement harness taxonomy enforcement and runbook marker parity contract checks
- Parent: #4192
- Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence
- Status: Implemented
- Priority: P1

## Problem Statement
The local full-stack integration policy checker validates runtime reason-taxonomy markers, but runbook marker parity enforcement is not part of the checker/lane contract path. Runbook drift detection currently lives in test-local helper logic instead of production contract enforcement.

## Scope
In scope:
- Implement runbook marker parity enforcement inside local full-stack policy checking flow.
- Wire contract lane to pass runbook path and surface deterministic runbook parity markers.
- Update release go/no-go governance docs with local full-stack parity contract markers.
- Extend docs-contract tests and shell tests to validate fail-closed behavior for runbook drift.

Out of scope:
- New runbook process automation.
- New reason taxonomies outside local full-stack integration parity.

## Acceptance Criteria
- AC-1: Given a valid local full-stack report and aligned runbook markers, when `check-policy` runs, then it returns `GO` and emits deterministic runbook parity markers.
- AC-2: Given runbook taxonomy marker drift, when `check-policy` runs with `--runbook-file`, then it fails closed with `local_full_stack_harness_taxonomy_mapping_drift_detected`.
- AC-3: Given runbook parity marker divergence, when `check-policy` or contract lane runs, then it fails closed with `runbook_marker_parity_mismatch`.
- AC-4: Given release governance docs, when docs-contract tests run, then the release checklist contains local full-stack harness parity guidance and deterministic reason taxonomy markers.

## Conformance Cases
- C-01 (Functional): Baseline `check-policy` with default runbook path emits runbook parity status `verified` and `local_full_stack_harness_runbook_reason_code=none`. (AC-1)
- C-02 (Regression): Tampered runbook taxonomy marker causes `check-policy` failure with `local_full_stack_harness_taxonomy_mapping_drift_detected`. (AC-2)
- C-03 (Regression): Tampered runbook reason marker causes `check-policy` failure with `runbook_marker_parity_mismatch`. (AC-3)
- C-04 (Integration): Contract lane passes runbook path into policy check and surfaces runbook parity markers in lane output/report. (AC-1, AC-3)
- C-05 (Docs Contract): `release_gonogo_checklist_docs` asserts local full-stack parity governance section and markers. (AC-4)
- C-06 (Docs Contract): `kolme_devnet_ops_docs` asserts runbook reason taxonomy version and reason code marker presence. (AC-1, AC-3)

## Success Metrics
- Deterministic fail-closed reason codes for runbook drift in both policy checker and contract lane integration.
- No regressions in existing local full-stack policy and contract-lane tests.
- Docs-contract tests pass with explicit marker-level assertions.
