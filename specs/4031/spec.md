# Issue #4031 Spec - CI Smoke Dependency Checker and Docs Threshold Parity

- Status: Reviewed
- Issue: #4031
- Parent: #4026
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
With advisory fixture/parser contracts now in place (`#4030`), CI smoke still lacks a deterministic
checker contract and docs parity enforcement for threshold governance markers.

## Scope
In scope:
- Add deterministic dependency CI smoke checker API contracts with fail-closed threshold behavior.
- Add checker contract tests and fixture-taxonomy superset regression coverage.
- Add docs threshold/remediation marker parity checks in `docs/ci/strategy.md`.
- Wire checker contract command into CI tools fast/full regression paths.

Out of scope:
- Local-heavy deep dependency scanning execution.
- External scanner ingestion and networked advisory feeds.

## Acceptance Criteria
- AC-1: Checker fails closed on threshold violations and unknown severity inputs.
- AC-2: Docs parity checks catch dependency threshold marker drift and missing remediation markers.
- AC-3: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): checker reason taxonomy/version markers remain deterministic.
- C-02 (Functional, AC-1): unknown severities and above-threshold advisories reject fail closed.
- C-03 (Integration, AC-1): fixture-derived threshold/advisory compositions map to deterministic
  allow/reject outcomes.
- C-04 (Regression, AC-1/AC-2): checker taxonomy remains fixture-superset and docs marker drift is
  rejected by docs-contract tests.
- C-05 (Performance, AC-3): checker evaluation path remains bounded for fast-gate usage.

## Success Metrics
- Dependency checker emits deterministic decisions and reason markers.
- Fixture threshold markers and checker taxonomy stay synchronized.
- CI docs-contract tests block threshold/remediation marker drift.
