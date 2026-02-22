# Spec: #5698 Activate Opt-In Live CLI-Scripted S-01 Driver Execution

- Issue: #5698
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
The CLI-scripted harness driver currently returns deterministic `"pass"` for all
scenario executions. To progress phase-3 live activation, S-01 should support a
real command execution path while preserving deterministic behavior by default.

## Scope
### In Scope
- Add opt-in live toggle for CLI-scripted driver execution.
- Implement S-01 live probe by invoking real `kamn-cli health`.
- Add injectable runner support for deterministic tests.
- Preserve default deterministic pass behavior when live mode is disabled.

### Out of Scope
- Live execution for scenarios other than S-01.
- CLI argument contract changes for harness `run`.
- MCP or SDK driver behavior changes.

## Acceptance Criteria
### AC-1 Live toggle gating
Given CLI-scripted driver execution,
When live toggle is disabled,
Then execution remains deterministic pass without invoking live command runner.

### AC-2 S-01 live command execution
Given live mode is enabled and scenario is `S-01`,
When `kamn-cli health` exits successfully,
Then scenario returns pass.
When command exits non-zero or fails to spawn,
Then scenario returns fail.

### AC-3 Non-S-01 compatibility
Given live mode is enabled and scenario is not `S-01`,
When executed,
Then deterministic pass behavior is preserved.

### AC-4 Regression stability
Given existing harness contract suites,
When CLI live toggle support is added,
Then existing harness tests remain green.

## Conformance Cases
- C-01 (AC-1): disabled-live CLI-scripted execution does not invoke runner and
  returns pass.
- C-02 (AC-2): enabled-live S-01 with successful runner returns pass.
- C-03 (AC-2): enabled-live S-01 with failing runner returns fail.
- C-04 (AC-3): enabled-live non-S-01 does not invoke runner and returns pass.
- C-05 (AC-4): `cargo test -p kamn-e2e-harness` passes.

## Success Metrics / Observable Signals
- CLI-scripted driver can emit real fail for S-01 when live mode is enabled.
- Default deterministic contract behavior remains unchanged.
- Harness crate tests continue to pass.
