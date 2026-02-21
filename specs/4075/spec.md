# Issue #4075 Spec — Retention Policy Fixture Matrix and Parser Helper Contracts

- Status: Reviewed
- Issue: #4075
- Parent: #4071
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Retention policy enforcement needs deterministic fixture coverage and parser helper contracts so
window-policy validation stays stable and fail closed before checker expansion work in follow-up
subtasks.

## Scope
In scope:
- Define a deterministic retention-policy fixture matrix with valid and invalid window cases.
- Add parser helper contract tests that validate metadata, row parsing, and expected decision
  mapping for each fixture row.
- Add docs markers in `docs/ops/configuration.md` and docs-parity tests.

Out of scope:
- Expanding runtime retention checker behavior beyond fixture/parser-helper contract coverage.
- Archive migration and multi-tier lifecycle automation.

## Acceptance Criteria
- AC-1: Fixture matrix includes deterministic metadata and both valid/invalid retention-window
  coverage.
- AC-2: Parser helper contracts are deterministic and test-backed for malformed and valid rows.
- AC-3: Integration contract verifies fixture rows map to expected status/reason outcomes.
- AC-4: `docs/ops/configuration.md` includes retention fixture/parser markers with parity tests.

## Conformance Cases
- C-01 (Functional, AC-1): fixture includes pass rows and fail rows for unknown domain and
  non-positive window.
- C-02 (Unit, AC-2): parser rejects malformed row column counts with deterministic error text.
- C-03 (Integration, AC-3): parsed fixture rows evaluate to expected `(status, reason)` outputs.
- C-04 (Regression, AC-4): docs and fixture metadata markers remain stable and case order is
  deterministic.

## Success Metrics
- Retention fixture rows parse deterministically across runs.
- Invalid rows fail closed with stable reason markers.
- Docs markers and tests stay synchronized with fixture contract surfaces.
