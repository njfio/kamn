# Issue #4080 Spec — Redaction Leak-Detection Policy Checker and Taxonomy/Docs Drift Guards

- Status: Reviewed
- Issue: #4080
- Parent: #4073
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Redaction validation lane artifacts require deterministic policy validation so leak markers,
reason-taxonomy metadata, and docs references cannot silently drift before release.

## Scope
In scope:
- Add a policy checker for local-heavy redaction lane reports.
- Add deterministic taxonomy/docs parity drift checks against `docs/ops/configuration.md` and
  `docs/ci/strategy.md`.
- Add checker contract tests and CI strategy marker assertions.

Out of scope:
- Redesigning runtime logging or sensitive-data processing internals.

## Acceptance Criteria
- AC-1: Checker fails closed on leak-marker and required-field violations.
- AC-2: Taxonomy and docs parity checks are deterministic and drift-detecting.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): valid baseline report passes checker with deterministic policy schema/taxonomy markers.
- C-02 (Functional, AC-1): tampered leak/profile marker fails checker with stable reason code.
- C-03 (Integration, AC-2): checker composes runner report + strategy docs + ops docs markers and passes when synchronized.
- C-04 (Regression, AC-2): taxonomy/docs drift fixtures fail checker with deterministic docs-parity reason.
- C-05 (Performance, AC-3): checker remains bounded low-cost.

## Success Metrics
- Policy checker emits deterministic pass/fail evidence with stable reason taxonomy.
- Drift in strategy/ops marker blocks is rejected pre-merge by tests.
- Checker runtime remains low-cost for fast-gate usage.
