# Spec — #4273 Subtask: Implement Protocol Taxonomy Enforcement and Runbook Marker Parity Checks

Status: Implemented
Priority: P1
Parent: #4267
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Contract lane orchestration must explicitly enforce checker taxonomy/runbook parity so drift is rejected before promotion decisions.

## Acceptance Criteria

AC-1: Lane enforces protocol taxonomy mapping parity markers.

AC-2: Lane rejects runbook parity drift with deterministic reason categories.

AC-3: Documentation and docs-contract tests encode the parity contract.

## Conformance Cases

- C-01 (AC-1): lane validates required runbook taxonomy markers.
- C-02 (AC-2): lane fails closed with deterministic taxonomy drift/parity mismatch reasons.
- C-03 (AC-3): deploy compatibility + release checklist docs include parity markers with docs-contract coverage.
