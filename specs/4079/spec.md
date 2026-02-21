# Issue #4079 Spec — Local-Heavy Redaction Validation Runner and Deterministic Artifact Schema

- Status: Reviewed
- Issue: #4079
- Parent: #4073
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Redaction confidence requires deterministic local-heavy validation artifacts that prove baseline
profiles pass and injected-leak profiles fail closed with stable leak markers.

## Scope
In scope:
- Add a local-heavy redaction validation lane runner and deterministic artifact schema markers.
- Support `baseline` and `injected-leak` profiles with stable decision/reason projection.
- Enforce explicit local-only opt-in for run mode.
- Add runner contract tests and ops-doc marker references.

Out of scope:
- Full enterprise DLP or NLP classification engine integration.

## Acceptance Criteria
- AC-1: Runner artifacts are deterministic and schema-valid.
- AC-2: Injected-leak profile fails closed with stable leak markers.
- AC-3: Local-heavy mode remains explicit opt-in and bounded.
- AC-4: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): baseline dry-run emits deterministic schema/taxonomy markers.
- C-02 (Functional, AC-2): `injected-leak` profile returns `status=fail`, `final_decision=NO-GO`, and leak reason code.
- C-03 (Integration, AC-3): run mode rejects missing local opt-in and succeeds with opt-in under `--ci-fast-gate FAIL`.
- C-04 (Regression, AC-1/AC-2): invalid profile fails closed with deterministic error marker.
- C-05 (Performance, AC-3/AC-4): baseline dry-run stays within bounded runtime budget.
- C-06 (Conformance, AC-1): ops docs include deterministic redaction runner schema/taxonomy/profile markers.

## Success Metrics
- Baseline profile remains GO with deterministic redaction-safe markers.
- Injected-leak profile always fails closed with stable taxonomy/reason markers.
- Local-heavy run mode cannot execute without explicit opt-in.
- Docs marker references stay deterministic and test-enforced.
