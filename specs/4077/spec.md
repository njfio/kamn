# Issue #4077 Spec — Deletion-Proof Fixture Set and Checker Behavior Contracts

- Status: Reviewed
- Issue: #4077
- Parent: #4072
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Deletion evidence must be validated with deterministic fixture classes and fail-closed checker
behavior so proof integrity drift is caught before production governance decisions.

## Scope
In scope:
- Add deterministic deletion-proof fixture matrix with valid and invalid proof classes.
- Add checker behavior contract tests for parser/evaluation outcomes across all fixture rows.
- Add `docs/ops/configuration.md` deletion-proof marker map and docs parity assertions.

Out of scope:
- External notarization workflows and legal-hold orchestration.

## Acceptance Criteria
- AC-1: Fixture matrix covers valid and invalid deletion-proof classes.
- AC-2: Checker behavior is deterministic across fixture rows.
- AC-3: Unit, Functional, Integration, and Regression tests exist and pass.
- AC-4: `docs/ops/configuration.md` includes deletion-proof marker map with docs-parity checks.

## Conformance Cases
- C-01 (Functional, AC-1): fixture includes pass and fail rows spanning missing subject, missing
  tombstone hash, invalid proof status, and hash mismatch.
- C-02 (Unit, AC-2): parser rejects malformed fixture rows deterministically.
- C-03 (Integration, AC-2/AC-3): checker evaluation for each fixture row matches expected
  `(status, reason)`.
- C-04 (Regression, AC-3/AC-4): schema/taxonomy markers and case order remain deterministic,
  and ops-doc marker map parity is enforced.

## Success Metrics
- Deletion-proof checks fail closed with deterministic reason markers.
- Fixture taxonomy and docs markers remain stable across updates.
- Contract tests guard against silent proof-class drift.
