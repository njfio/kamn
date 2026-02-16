# Spec: Issue #4451

Status: Implemented
Issue: #4451
Parent: #4446
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

Runtime extraction policy checks currently emit raw failure lists without a normalized
taxonomy reason value, and parity evidence outputs are not explicitly normalized as a
stable contract marker. Review and regression workflows need deterministic reason mapping
and deterministic parity evidence-output markers to detect drift with low ambiguity.

## Scope

In scope:
- Deterministic runtime extraction reason mapping in the local full-stack integration
  policy contract.
- Stable parity evidence-output normalization markers in runtime extraction outputs.
- Conformance coverage for reason-output mapping and parity output normalization.
- Runtime architecture docs updates for extraction reason taxonomy references.

Out of scope:
- Runtime execution flow rewrites.
- Changes to external protocol/wire formats.
- CI workflow topology changes.

## Acceptance Criteria

AC-1:
Given a local full-stack integration policy validation failure, when policy checks fail,
then the policy output must include a deterministic normalized reason value mapped from
failure checks.

AC-2:
Given runtime extraction parity outputs, when lane and policy outputs are emitted, then
parity evidence-output markers must be deterministic and stable across runs.

AC-3:
Given integration and docs contract checks, when targeted tests run, then tests must
validate reason-output mapping and parity marker normalization fail-closed behavior.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
  - Expectation: policy output includes deterministic `reason_codes_value` mapping and
    deterministic fail-closed mapping for parity extraction evidence drift.

- C-02 (AC-2, Integration/Conformance):
  - Test: `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
  - Expectation: contract-lane output and reports include deterministic parity
    evidence-output normalization markers.

- C-03 (AC-3, Regression/Conformance):
  - Test: `cargo test -p kamn-core --test runtime_architecture_docs`
  - Expectation: runtime docs include deterministic runtime extraction reason mapper and
    parity evidence-output marker references.

## Success Metrics / Observable Signals

- Policy and contract-lane outputs expose stable normalized reason/evidence markers.
- New conformance checks fail closed when reason-output mapping or parity markers drift.
- Targeted runtime/docs test lanes remain deterministic on repeated runs.
