# Spec — Issue #4827

- Title: Subtask: build `declarative_policy_checker.py` and declarative policy schema contracts
- Parent: Parent task `#4815`
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Add a reusable Python checker that evaluates policy checks against report JSON payloads and emits deterministic `GO/NO-GO` outputs with stable reason taxonomy markers.

## Problem Statement

Contract lane wrappers currently rely on script-specific ad hoc policy checks. This blocks broad shell LOC reduction and reuse because behavior and failure semantics are not centralized behind one declarative evaluator contract.

## Scope

In scope:
- `scripts/framework/declarative_policy_checker.py` implementation
- deterministic policy schema validation (`kamn.framework.declarative-policy.v1`)
- deterministic output schema emission (`kamn.framework.declarative-policy-report.v1`)
- unit tests and contract-lane shell tests for checker behavior
- integration into `scripts/framework/test_contract_framework.sh`

Out of scope:
- broad migration of all lane policies to declarative checker in this issue
- protocol/schema changes outside checker policy/output versions above

## Acceptance Criteria

- AC-1: Given a valid policy and report file, when checker evaluation succeeds with no failed checks, then checker exits `0`, prints `status=ok`, `final_decision=GO`, and writes output JSON with schema version `kamn.framework.declarative-policy-report.v1`.
- AC-2: Given a valid policy and a report that violates one or more checks, when `--expected-final-decision GO` is supplied, then checker fails closed (non-zero), still emits deterministic markers (`status=ok`, `final_decision=NO-GO`, ordered reason codes), and includes stable `reason_key`.
- AC-3: Given an invalid policy schema payload, when checker runs, then checker exits non-zero with deterministic validation failure text and no silent fallback behavior.
- AC-4: Checker behavior is covered by deterministic tests wired into framework regression entrypoint (`test_contract_framework.sh`).

## Conformance Cases

- C-01 (AC-1, Functional/Conformance): `test_main_writes_output_json_and_ci_fast_gate` validates success path output file and GO markers.
- C-02 (AC-2, Functional/Conformance): `test_main_raises_when_expected_final_decision_mismatches` plus `test_declarative_policy_checker_contract.sh` validates fail-closed mismatch with NO-GO reason output.
- C-03 (AC-3, Unit/Conformance): `test_validate_policy_rejects_missing_expected_for_non_exists_check` and invalid-schema shell case validate deterministic validation rejection.
- C-04 (AC-4, Integration): `scripts/framework/test_contract_framework.sh` runs checker unit and shell contract suites.

## Success Metrics / Signals

- Checker produces deterministic key/value outputs and output JSON fields for both GO and NO-GO decisions.
- `scripts/framework/test_contract_framework.sh` remains green with checker coverage included.
- No new shell wrapper family introduced for this capability; behavior is consolidated in one Python module + tests.
