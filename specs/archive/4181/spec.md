# Spec — Issue #4181

- Title: implement deterministic compatibility matrix checker and fail-closed reason outputs
- Parent: #4176
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Compatibility decisions need a deterministic checker that validates runtime/schema/failure-taxonomy
markers across version and fork policy reports, with explicit fail-closed reason mapping.

## Scope

In scope:
- implement compatibility marker matrix checker with deterministic reason taxonomy,
- integrate checker into version-compatibility contract lane,
- update required docs and docs-contract tests.

Out of scope:
- redesign of version/fork evidence generators,
- multi-chain compatibility framework expansion.

## Acceptance Criteria

- AC-1: checker emits deterministic pass/fail outputs and taxonomy markers.
- AC-2: marker mismatches fail closed with deterministic reason codes.
- AC-3: version-compatibility contract lane includes checker gate.
- AC-4: docs and docs-contract tests capture checker command/output markers.

## Conformance Cases

- C-01: baseline matrix reports produce `final_decision=GO` and `reason_codes_value=none`.
- C-02: schema/taxonomy/csv/rehearsal marker mismatches produce deterministic `NO-GO` reasons.
- C-03: contract lane continues to pass with checker integrated.
- C-04: ops/release docs include compatibility matrix checker markers and fail-closed reason codes.

## Success Metrics / Signals

- checker is executable and wired into contract lane,
- mismatch reasons are deterministic and ordered,
- docs contracts fail closed on marker drift.
