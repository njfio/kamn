# Spec — #4196 Subtask: Deterministic Harness Checker Outputs and Fail-Closed Reason Mapping

Status: Implemented
Priority: P1
Parent: #4191
Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence

## Problem Statement
The full I/O scenario matrix policy checker does not yet publish the same deterministic reason-taxonomy output shape used by other release gates, and release checklist docs do not yet define its required reason outputs.

## Scope
In scope:
- Add deterministic checker output markers for full I/O scenario matrix policy evaluation.
- Add explicit fail-closed reason mapping output fields for pass/fail paths.
- Add release checklist guidance for harness marker checker reason outputs.
- Add docs-contract coverage for the new checklist section.

Out of scope:
- Runtime/transport feature behavior changes outside checker output semantics.
- CI/CD topology changes.

## Acceptance Criteria
AC-1 (Given/When/Then):
- Given the same full I/O report input and policy options,
- When the policy checker runs repeatedly,
- Then deterministic reason-taxonomy output markers and reason-code values remain stable.

AC-2 (Given/When/Then):
- Given a full I/O report with harness-marker or parity drift,
- When policy checking runs,
- Then it fails closed with deterministic reason mapping markers and stable fail codes.

AC-3 (Given/When/Then):
- Given release go/no-go checklist documentation,
- When docs-contract tests run,
- Then required full-stack harness checker reason-output markers and regression policy text are present.

## Conformance Cases
- C-01 (AC-1, Functional/Regression): run checker twice against the same passing report and assert stable deterministic reason marker output.
- C-02 (AC-2, Regression): tamper required harness marker and assert fail-closed reason mapping marker.
- C-03 (AC-2, Regression): tamper dry-run/run parity marker and assert fail-closed reason mapping marker.
- C-04 (AC-3, Docs): release checklist includes full-stack harness checker commands, taxonomy marker, reason-codes CSV/value semantics, and regression policy.

## Success Metrics / Signals
- Policy checker tests pass with deterministic marker assertions on pass/fail paths.
- Release checklist docs-contract tests pass with new full-stack harness section.
