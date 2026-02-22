# Spec: #5696 Activate Opt-In Live SDK-Direct S-01 Driver Execution

- Issue: #5696
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-e2e-harness` scenario drivers currently return deterministic `"pass"` for
all scenarios, which prevents real execution in `sdk-direct` mode. The harness
needs a low-risk activation path that allows genuine live validation work to
start without breaking existing deterministic contract tests.

## Scope
### In Scope
- Add an opt-in live toggle for SDK-direct driver execution.
- Implement real S-01 execution path in live mode using `kamn-agent-lib`
  discovery signals (identity + health).
- Keep deterministic pass behavior as default when live mode is not enabled.
- Add tests validating toggle gating and S-01 pass/fail mapping with test probes.

### Out of Scope
- Live execution for non-S-01 scenarios.
- CLI-scripted or MCP driver live execution.
- Changes to run output JSON contract shape.

## Acceptance Criteria
### AC-1 Live toggle gating
Given SDK-direct driver execution,
When live toggle is disabled,
Then scenario execution remains deterministic pass without invoking live probes.

### AC-2 S-01 live execution
Given SDK-direct live mode is enabled and scenario is `S-01`,
When live probe succeeds,
Then scenario returns pass.
When live probe fails,
Then scenario returns fail.

### AC-3 Non-S-01 compatibility
Given SDK-direct live mode is enabled and scenario is not `S-01`,
When executed,
Then deterministic pass behavior is preserved.

### AC-4 Regression stability
Given existing harness contract suites,
When the SDK-direct live toggle feature is added,
Then existing contract suites remain green.

## Conformance Cases
- C-01 (AC-1): disabled-live SDK-direct execution does not invoke probe and
  returns pass.
- C-02 (AC-2): enabled-live S-01 with successful probe returns pass.
- C-03 (AC-2): enabled-live S-01 with failing probe returns fail.
- C-04 (AC-3): enabled-live non-S-01 does not invoke probe and returns pass.
- C-05 (AC-4): `cargo test -p kamn-e2e-harness` passes.

## Success Metrics / Observable Signals
- SDK-direct driver can emit real fail for S-01 in live mode.
- Deterministic contract behavior remains unchanged when live mode is off.
- Harness crate tests continue to pass.
