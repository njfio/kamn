# Spec — #4197 Subtask: Red Tests for Harness Reason-Taxonomy Drift and Runbook Marker Divergence

Status: Implemented
Priority: P1
Parent: #4192
Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence

## Problem Statement
Local full-stack harness taxonomy markers and runbook marker declarations can drift independently, which risks ambiguous remediation when contract checks fail.

## Scope
In scope:
- Add red-path tests for local full-stack harness taxonomy drift.
- Add red-path tests for runbook marker divergence in local full-stack harness governance docs.
- Add `docs/deploy/kolme_devnet_ops.md` section defining harness taxonomy and runbook parity markers.
- Add docs-contract assertions for the new runbook section.

Out of scope:
- Runtime/transport feature redesign.
- New CI workflow topology.

## Acceptance Criteria
AC-1 (Given/When/Then):
- Given local full-stack policy input with tampered taxonomy marker values,
- When policy checker tests run,
- Then tests fail closed with deterministic taxonomy mismatch reasons.

AC-2 (Given/When/Then):
- Given runbook marker divergence in `docs/deploy/kolme_devnet_ops.md`,
- When contract-lane tests validate runbook marker parity,
- Then tests fail with deterministic runbook divergence reasons.

AC-3 (Given/When/Then):
- Given deploy runbook docs contracts,
- When docs-contract tests run,
- Then full-stack harness taxonomy/runbook marker section and required markers are present.

## Conformance Cases
- C-01 (AC-1, Regression): tamper `runtime_phase_parity_reason_taxonomy_version` and assert deterministic mismatch reason.
- C-02 (AC-1, Regression): tamper `runtime_phase_parity_reason_codes_csv` and assert deterministic mismatch reason.
- C-03 (AC-1, Regression): tamper `runtime_module_boundary_parity_reason_codes_csv` and assert deterministic mismatch reason.
- C-04 (AC-2, Regression): tamper runbook marker declarations and assert deterministic runbook marker divergence reason.
- C-05 (AC-3, Docs): docs include full-stack harness taxonomy/runbook marker contract section and regression anchors.

## Success Metrics / Signals
- Updated local full-stack policy/contract-lane shell tests pass with deterministic taxonomy/runbook drift assertions.
- `kolme_devnet_ops` docs-contract tests pass with required marker coverage.
