# Spec: Issue #5891 - Expand Default Panic-Path Audit Coverage to kamn-agent-lib

- Issue: #5891
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The default production panic-path audit root list omits `crates/kamn-agent-lib/src`, leaving part of the SDK/agent-lib runtime surface outside default contract evidence.

## Scope
In scope:
- Add `crates/kamn-agent-lib/src` to `DEFAULT_RUNTIME_ROOTS` in `scripts/ci/check_no_production_expect.py`.
- Re-validate default checker and checker regression suite.

Out of scope:
- Checker taxonomy semantics changes.
- Runtime behavior changes in Rust crates.

## Acceptance Criteria
### AC-1 Agent-lib included in default checker roots
Given `scripts/ci/check_no_production_expect.py`,
When reading `DEFAULT_RUNTIME_ROOTS`,
Then `crates/kamn-agent-lib/src` is present.

### AC-2 Default checker remains green
Given the updated root set,
When `scripts/ci/check_no_production_expect.sh` runs,
Then it exits successfully with `violation_count=0`.

### AC-3 Checker regression suite remains green
Given the updated root set,
When `scripts/ci/test_check_no_production_expect.sh` runs,
Then it exits successfully.

### AC-4 Agent-lib scoped checker remains green
Given `crates/kamn-agent-lib/src`,
When `python3 scripts/ci/check_no_production_expect.py --root crates/kamn-agent-lib/src` runs,
Then it exits successfully with `violation_count=0`.

## Conformance Cases
- C-01 (Functional, AC-1): static scan confirms agent-lib root is in default root tuple.
- C-02 (Conformance, AC-2): wrapper checker invocation passes (`violation_count=0`).
- C-03 (Regression, AC-3): checker test suite invocation passes.
- C-04 (Conformance, AC-4): agent-lib scoped checker invocation passes.

## Success Metrics / Observable Signals
- Default panic-path checker output remains `status=ok` with zero violations.
- Agent-lib root covered by default checker configuration.
