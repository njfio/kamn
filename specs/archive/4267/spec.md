# Spec — #4267 Task: Enforce Protocol Taxonomy and Runbook Marker Parity Checks

Status: Implemented
Priority: P1
Parent: #4264
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Service API axum ingress protocol taxonomy markers are deterministic in checker outputs, but runbook compatibility docs must remain in strict parity with those markers. Drift between checker taxonomy and runbook markers can break deterministic remediation and promotion decisions.

## Scope

In scope:
- Enforce protocol taxonomy marker parity against runbook compatibility documentation.
- Fail closed on taxonomy drift and runbook marker divergence with deterministic reason outputs.
- Add regression coverage for drift/divergence behavior.
- Update runbook/checklist docs and docs-contract tests.

Out of scope:
- API endpoint redesign.
- Runtime networking behavior changes.

## Acceptance Criteria

AC-1: Protocol taxonomy mapping remains deterministic and parity-checked against runbook markers.

AC-2: Runbook parity checks fail closed on marker drift/divergence with deterministic reasons.

AC-3: Regression tests preserve taxonomy/runbook alignment behavior.

AC-4: Functional/Integration/Regression/Conformance checks pass for this lane.

## Conformance Cases

- C-01 (AC-1, Functional): service api axum contract lane validates expected protocol taxonomy marker set and runbook parity markers.
- C-02 (AC-2, Regression): taxonomy marker drift in runbook fails closed with deterministic `protocol_taxonomy_mapping_drift_detected`.
- C-03 (AC-2, Regression): runbook marker divergence fails closed with deterministic `runbook_marker_parity_mismatch`.
- C-04 (AC-3, Regression): repeated regression checks preserve deterministic reason output for drift/divergence.
- C-05 (AC-4, Conformance): docs and docs-contract tests enforce runbook/checklist parity markers.

## Success Signals

- Contract lane exits non-zero on taxonomy/runbook drift.
- Failure outputs include deterministic reason categories.
- Runbook and checklist docs contain parity marker contracts and stay synchronized with tests.
