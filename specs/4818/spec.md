# Spec — Issue #4818

- Title: Task: enforce shell-surface process contracts in issue templates, specs, and PR requirements
- Parent: Parent story: #4810
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Make shell-surface containment mandatory in future work intake/execution/merge processes.

## Problem Statement

Current script surface includes large duplicated boilerplate and uneven governance boundaries that increase maintenance burden.

## Scope

In scope:
- phase-aligned implementation and regression checks
- deterministic reason-taxonomy and compatibility markers where applicable
- bounded CI/runtime governance requirements

Out of scope:
- unrelated runtime feature delivery
- non-deterministic policy behavior

## Acceptance Criteria

- AC-1: Issue templates and contributing contracts require shell LOC impact and mitigation sections.
- AC-2: PR template and docs-contract tests enforce shell-surface accountability markers.
- AC-3: Future script-heavy changes cannot bypass process guardrails.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
