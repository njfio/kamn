# Spec: #5688 Decompose `kamn-e2e-harness` Run-Contract Monolith

- Issue: #5688
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`crates/kamn-e2e-harness/src/lib.rs` currently centralizes run-contract execution,
runtime probe helpers, lifecycle aggregation, and deterministic output assembly in
one large module. This makes maintenance and review expensive.

## Scope
### In Scope
- Extract run-contract execution and helper logic from `lib.rs` into dedicated
  submodule file(s).
- Preserve current public API signatures and deterministic run-output contracts.
- Keep existing tests green and add parity assertions only if needed.

### Out of Scope
- New runtime capabilities.
- Changes to run/verify command semantics.
- CLI argument surface changes.

## Acceptance Criteria
### AC-1 Behavior parity
Given existing harness contract tests,
When refactor is applied,
Then run-output markers and verify behavior remain unchanged.

### AC-2 Structural decomposition
Given current monolithic `lib.rs`,
When refactor completes,
Then run-contract logic is moved into submodule(s) and `lib.rs` shrinks materially.

### AC-3 Regression confidence
Given targeted harness suites,
When verification runs,
Then command and manifest contract suites pass without new failures.

## Conformance Cases
- C-01 (AC-1): `command_contract` suite passes including runtime marker checks.
- C-02 (AC-2): `execute_run_contract` is provided via extracted module wiring.
- C-03 (AC-3): harness crate tests remain green after refactor.

## Success Metrics
- `crates/kamn-e2e-harness/src/lib.rs` LOC reduced.
- `cargo test -p kamn-e2e-harness --test command_contract` passes.
- `cargo test -p kamn-e2e-harness --test mode_scenario_manifest_contract` passes.
