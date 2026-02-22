# Spec: #5666 Enable cargo-mutants In-Diff Gate for Portable-Agent Slices

- Issue: #5666
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P2

## Problem Statement
Portable-agent task slices currently cannot provide mutation-test evidence because
`cargo-mutants` is unavailable in the standard local environment. This leaves the
Mutation tier in AGENTS.md under-enforced for R52 slices.

## Scope
### In Scope
- Establish deterministic local invocation guidance for `cargo mutants --in-diff`.
- Document install + execution + fallback behavior in CI strategy docs.
- Capture red/green command evidence for missing-tool -> available-tool transition.

### Out of Scope
- Expanding mutation policy outside portable-agent slices.
- Deep mutation campaigns across the full workspace.
- New CI workflow wiring (explicitly deferred in this task).

## Acceptance Criteria
### AC-1 Tooling availability transition
Given a baseline environment without cargo-mutants,
When setup steps are applied,
Then `cargo mutants --in-diff` is invokable from repo root.

### AC-2 Deterministic gate guidance
Given portable-agent slice validation,
When engineers run the mutation tier,
Then docs provide deterministic install and invocation commands for in-diff mutation runs.

### AC-3 Fallback behavior documentation
Given environments where cargo-mutants cannot be installed,
When mutation evidence is collected,
Then docs describe the required fallback evidence and follow-up behavior.

## Conformance Cases
- C-01 (AC-1): RED evidence shows `cargo mutants` missing; GREEN evidence shows invokable command.
- C-02 (AC-2): `docs/ci/strategy.md` contains explicit in-diff mutation gate commands for portable-agent slices.
- C-03 (AC-3): `docs/ci/strategy.md` contains explicit fallback/waiver behavior requirements.

## Success Metrics
- `cargo mutants --version` returns a version string in the implementation environment.
- `cargo mutants --in-diff --list` executes from repo root without command-not-found failure.
- Mutation-gate guidance appears in `docs/ci/strategy.md` and is reviewable in PR diff.
