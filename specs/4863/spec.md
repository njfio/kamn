# Spec — Issue #4863

- Title: Story: institutionalize permanent CI and process contracts for shell-surface sustainment
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

As a reviewer, I need permanent shell-surface governance hooks in CI and process templates so future work cannot regress maintainability or shell:Rust ratio.

## Problem Statement

Without sustained ratchets and mandatory declarations, shell surface can regrow despite one-time reduction efforts.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: CI policy checks enforce shell-surface budget and ratio trajectory with deterministic reason markers.
- AC-2: Issue/PR templates and docs-contract tests require shell delta + mitigation declarations.
- AC-3: New script-surface growth cannot merge without explicit mitigation path.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
