# Spec — #4228 Subtask: Implement Admission Taxonomy Enforcement and Runbook Marker Parity Checks

Status: Implemented
Priority: P1
Parent: #4222
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
After red tests, production scripts must enforce admission decision taxonomy output and runbook parity checks to keep overload governance deterministic.

## Scope
In scope:
- Implement admission decision taxonomy marker emission in validation reports.
- Implement policy checker enforcement + deterministic mismatch reasons.
- Implement contract-lane runbook parity marker checks.

Out of scope:
- Admission algorithm redesign.
- CI topology changes.

## Acceptance Criteria
AC-1: Validation + policy outputs include deterministic admission decision taxonomy markers.
AC-2: Policy checker fails closed on admission decision taxonomy drift.
AC-3: Contract lane fails closed on runbook admission marker parity drift/divergence.
AC-4: Docs and docs-contract tests reflect and enforce the marker contract.

## Conformance Cases
- C-01 (AC-1, Functional): baseline validation and policy output include taxonomy version/csv + accept/defer/reject markers.
- C-02 (AC-2, Regression): tampered admission decision taxonomy version fails policy.
- C-03 (AC-2, Regression): tampered admission decision taxonomy csv fails policy.
- C-04 (AC-3, Regression): runbook taxonomy marker drift fails lane.
- C-05 (AC-3, Regression): runbook marker divergence fails lane.
- C-06 (AC-4, Docs): docs-contract tests assert new taxonomy/parity markers.
