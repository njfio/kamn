# Issue #4030 Spec - Advisory Parser and Dependency-Threshold Fixture Contracts

- Status: Reviewed
- Issue: #4030
- Parent: #4026
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Dependency-risk CI smoke checks need deterministic advisory parsing and threshold fixtures before
checker wiring can enforce policy in fast-gate without brittle or ambiguous severity handling.

## Scope
In scope:
- Add deterministic dependency advisory fixture matrix with severity coverage and threshold metadata.
- Add parser/helper contract tests for advisory rows and threshold metadata parsing.
- Add threshold-mapping contract tests that deterministically map advisory severity to pass/fail
  policy outcomes.
- Add CI strategy marker documentation and docs-parity assertions.

Out of scope:
- CI workflow wiring for dependency smoke checker command execution.
- Local-heavy deep dependency scan execution and policy checks.

## Acceptance Criteria
- AC-1: Fixture matrix includes deterministic schema/taxonomy metadata plus advisory severity rows
  for pass/fail outcomes.
- AC-2: Parser helper contracts deterministically reject malformed rows and invalid threshold
  metadata.
- AC-3: Integration contract validates advisory severity to threshold outcome mapping for each
  fixture case.
- AC-4: `docs/ci/strategy.md` includes dependency advisory fixture/threshold markers and guard
  commands, enforced by docs-contract tests.

## Conformance Cases
- C-01 (Functional, AC-1): fixture includes low/moderate pass cases and high/critical/unknown fail
  cases with deterministic reason codes.
- C-02 (Unit, AC-2): parser rejects malformed advisory case column counts with deterministic error
  text.
- C-03 (Integration, AC-3): parsed advisory rows evaluate to expected `(status, reason)` values
  under configured maximum severity threshold.
- C-04 (Regression, AC-4): docs and fixture metadata markers remain stable and fixture case order
  is deterministic.
- C-05 (Performance, AC-3): fixture parse/evaluate path remains bounded for CI smoke usage.

## Success Metrics
- Advisory severity parsing and threshold evaluation remain deterministic across runs.
- Threshold drift and malformed rows fail closed with stable reason markers.
- Strategy docs markers remain synchronized with fixture/test contract surfaces.
