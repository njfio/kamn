# Spec: #5682 Scenario Contract Projection in E2E Harness Run Output

- Issue: #5682
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-e2e-harness` scenario modules currently expose only `{id, name, priority}` metadata. The `run` command output therefore cannot surface which PRD-defined scenario steps, verifiable outputs, and pass criteria are selected for execution, reducing traceability between PRD section 7 and runtime payloads.

## Scope
### In Scope
- Extend scenario model with deterministic contract metadata:
  - steps
  - verifiable outputs
  - pass criteria
- Populate PRD-backed contract metadata for P0 scenarios (S-01..S-06).
- Ensure remaining scenarios return non-empty deterministic contract metadata.
- Project selected scenario contracts into `execute_run_contract` output in selected order.
- Add/update tests for inventory and run-output contract projection.

### Out of Scope
- Real network execution of scenario operations.
- Full evidence artifact generation per scenario.
- New CLI flags or command-surface changes.

## Acceptance Criteria
### AC-1 Scenario model enrichment
Given scenario definitions are loaded,
When `all_scenarios()` is called,
Then each scenario includes non-empty step/output/pass-criteria contract metadata.

### AC-2 P0 PRD alignment
Given P0 scenarios S-01 through S-06,
When scenario definitions are inspected,
Then contract metadata reflects PRD section-7 scenario steps and outputs.

### AC-3 Run-output projection
Given `execute_run_contract` with selected scenarios,
When run output is generated,
Then output includes `scenario_contracts` in the same selected order with scenario metadata + contract details.

### AC-4 Regression compatibility
Given existing command and manifest contract suites,
When tests run,
Then existing behavior remains green with updated scenario contract expectations.

## Conformance Cases
- C-01 (AC-1): all scenarios expose non-empty `steps`, `verifiable_outputs`, and `pass_criteria`.
- C-02 (AC-2): S-01..S-06 scenario contracts include PRD-aligned key entries.
- C-03 (AC-3): `run` output includes ordered `scenario_contracts` matching selected scenario IDs.
- C-04 (AC-4): existing `command_contract` and `mode_scenario_manifest_contract` suites remain green.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test mode_scenario_manifest_contract` passes with scenario contract assertions.
- `cargo test -p kamn-e2e-harness --test command_contract` passes with scenario contract projection coverage.
- `cargo test -p kamn-e2e-harness` passes with no regressions.
