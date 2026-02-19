# Spec — #4222 Task: Enforce Admission Decision Taxonomy and Runbook Marker Parity Under Overload

Status: Implemented
Priority: P1
Parent: #4219
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
Service API axum ingress contracts currently verify admission saturation and budget markers but do not expose an explicit admission decision taxonomy contract for `accept`/`defer`/`reject` with runbook parity validation. This creates drift risk between checker output and operator runbook guidance during overload handling.

## Scope
In scope:
- Add deterministic admission decision taxonomy markers (`accept`, `defer`, `reject`) to service-api axum validation/policy outputs.
- Enforce policy fail-closed behavior for admission decision taxonomy drift.
- Extend contract-lane runbook parity requirements to include admission decision taxonomy markers.
- Update release/checklist/CI/runbook docs and docs-contract tests.

Out of scope:
- Redesigning ingress admission engine behavior.
- Changing runtime queue/concurrency defaults.
- Incident workflow tooling changes.

## Acceptance Criteria
AC-1 (Given/When/Then):
- Given a valid service-api axum ingress summary report,
- When the policy checker validates the report,
- Then the output includes deterministic admission decision taxonomy markers for `accept`, `defer`, and `reject`, and returns `final_decision=GO`.

AC-2 (Given/When/Then):
- Given a tampered admission decision taxonomy field in the summary report,
- When policy validation runs,
- Then validation fails closed with deterministic mismatch reason codes.

AC-3 (Given/When/Then):
- Given a runbook missing admission decision taxonomy markers,
- When the contract lane validates runbook parity,
- Then the lane fails closed with deterministic taxonomy/parity drift reasons.

AC-4 (Given/When/Then):
- Given CI/docs/release operator documents,
- When docs-contract tests run,
- Then all required admission decision taxonomy and runbook-parity markers are present and synchronized.

## Conformance Cases
- C-01 (AC-1, Functional): baseline policy output contains deterministic admission decision taxonomy version/csv and `accept/defer/reject` status markers.
- C-02 (AC-1, Integration): contract-lane output/report propagate admission decision taxonomy markers.
- C-03 (AC-2, Regression): tampered admission decision taxonomy version fails with deterministic mismatch reason.
- C-04 (AC-2, Regression): tampered admission decision reason-codes csv fails with deterministic mismatch reason.
- C-05 (AC-3, Regression): runbook taxonomy marker drift fails lane with deterministic taxonomy drift reason.
- C-06 (AC-3, Regression): runbook marker divergence fails lane with deterministic parity mismatch reason.
- C-07 (AC-4, Docs): docs-contract tests assert admission decision taxonomy/runbook markers in CI strategy, release checklist, and runbook docs.

## Success Metrics / Signals
- Service-api axum policy checker and contract-lane tests pass with new deterministic markers.
- Runbook drift/divergence fixtures fail deterministically.
- Updated docs-contract tests pass without flaky ordering or output variance.
