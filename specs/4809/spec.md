# Spec — Issue #4809

- Title: Story: consolidate policy-checker/manifests into declarative generated architecture
- Parent: Parent epic: #4806
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Execute phases 6-7 by moving eligible policy checks to declarative configuration and generating manifests/symlinks from a registry source of truth.

## Problem Statement

Large amounts of Python and static manifest shell wiring are repetitive and difficult to maintain consistently across lane growth.

## Scope

In scope:
- declarative policy-checker framework and migrations
- registry-driven manifest generation and consistency checks
- wrapper simplification aligned with generated metadata

Out of scope:
- removing complex domain-specific Python checks requiring bespoke logic
- changing release governance semantics outside declared migration scope

## Acceptance Criteria

- AC-1: Eligible policy-checker files migrate to declarative form with deterministic output parity.
- AC-2: Manifest/entrypoint generation removes manual static drift points and adds verification checks.
- AC-3: Generated artifacts remain contract-compatible with existing lane invocation surfaces.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
