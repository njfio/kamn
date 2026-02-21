# Issue #4078 Spec — Deletion Governance Docs/Runbook Parity and Remediation Markers

- Status: Implemented
- Issue: #4078
- Parent: #4072
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Deletion-governance contracts can drift if CI strategy and ops runbook markers diverge or if
reason-code remediation mappings are incomplete.

## Scope
In scope:
- Add deletion docs/runbook parity markers to `docs/ci/strategy.md`.
- Add deterministic remediation marker map entries for each deletion-proof reason code.
- Add parity/remediation drift tests in `ci_strategy_docs` (and ops-doc assertions as needed).

Out of scope:
- Broad docs migration outside deletion governance marker scope.

## Acceptance Criteria
- AC-1: Docs parity checks fail closed on deletion marker drift.
- AC-2: Remediation markers remain synchronized with deletion checker reason codes.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): strategy docs section includes deletion parity markers, guard commands,
  and regression marker.
- C-02 (Integration, AC-1/AC-2): strategy markers match ops-doc and fixture taxonomy markers.
- C-03 (Regression, AC-2): every deletion reason code has deterministic remediation entries in
  strategy and ops docs.
- C-04 (Unit/Regression, AC-3): reason-code parser/helper loop checks remain deterministic.

## Success Metrics
- Deletion marker drift between strategy and ops docs is detected by tests.
- Remediation map coverage remains complete for all deletion reason codes.
- Docs-only governance checks remain low-cost and deterministic.
