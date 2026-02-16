# Spec: Issue #4455

Status: Implemented
Issue: #4455
Parent: #4448
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

Panic-path replacement checks need deterministic reason taxonomy and normalized runtime evidence
outputs so promotion/go-no-go policy checks are auditable and machine-parse stable.

## Scope

In scope:
- Deterministic reason taxonomy/version/value/class markers for production panic checker output.
- Normalized runtime evidence output markers for checker reports.
- Release go/no-go checklist documentation updates for panic-free taxonomy references.
- Contract tests for checker output and docs markers.

Out of scope:
- New panic/error framework redesign.
- Replacing existing promotion workflow architecture.

## Acceptance Criteria

AC-1:
Given panic-replacement checker outputs, when pass/fail/configuration paths execute, then
`reason_taxonomy_version`, `reason_codes_csv`, `reason_codes_value`, and deterministic
`reason_class` markers are emitted.

AC-2:
Given checker report output, when violations are present/absent, then normalized runtime evidence
markers are emitted deterministically (`runtime_panic_replacement_evidence_*`).

AC-3:
Given docs contracts, when release/security docs tests run, then panic-free taxonomy and runtime
 evidence markers are present in `docs/security/secure-coding.md` and
`docs/foundation/release-gonogo-checklist.md`.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_no_production_expect.sh`
  - Expectation: deterministic reason taxonomy/value/class markers for pass/fail/configuration cases.

- C-02 (AC-2, Integration/Conformance):
  - Test: `bash scripts/ci/test_check_no_production_expect.sh`
  - Expectation: deterministic normalized runtime evidence output markers are emitted.

- C-03 (AC-3, Regression/Conformance):
  - Tests:
    - `cargo test -p kamn-core --test secure_coding_docs`
    - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: docs include panic-free taxonomy and runtime evidence references.

## Success Metrics / Observable Signals

- Checker emits deterministic taxonomy/value/class outputs for all paths.
- Runtime evidence markers stay stable across repeated runs.
- Docs contracts fail closed if panic taxonomy/evidence references drift.
