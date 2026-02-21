# Issue #4076 Spec — Fail-Closed Retention Checker and Deterministic Taxonomy Contracts

- Status: Reviewed
- Issue: #4076
- Parent: #4071
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Retention policy violations must emit deterministic fail-closed reasons so operators can audit and
enforce lifecycle-window behavior without ambiguity.

## Scope
In scope:
- Add deterministic retention checker APIs with explicit fail-closed reason taxonomy markers.
- Add checker contract tests (unit/functional/integration/regression), including fixture-taxonomy
  superset validation against `#4075` fixture reason markers.
- Add `docs/ci/strategy.md` retention-checker taxonomy section and docs-parity tests.

Out of scope:
- Multi-tier archival actions and broader lifecycle orchestration changes.

## Acceptance Criteria
- AC-1: Checker rejects invalid inputs and expired records with deterministic fail-closed reasons.
- AC-2: Checker reason taxonomy version and reason-code markers remain deterministic.
- AC-3: Checker taxonomy remains a superset of fixture taxonomy markers from `#4075`.
- AC-4: `docs/ci/strategy.md` contains retention-checker taxonomy markers and guard commands,
  enforced by docs-contract tests.

## Conformance Cases
- C-01 (Functional, AC-1): unknown domains and non-positive windows reject fail closed.
- C-02 (Integration, AC-1): expired records reject with stable expired reason; non-expired inputs
  allow.
- C-03 (Regression, AC-2/AC-3): checker taxonomy version/codes remain stable and include all fixture
  reason markers.
- C-04 (Regression, AC-4): strategy docs parity markers remain synchronized with checker contracts.

## Success Metrics
- Retention checker behavior is deterministic for allow/reject boundaries.
- Taxonomy strings are stable and docs-validated.
- Fixture and checker reason taxonomies remain coherently aligned.
