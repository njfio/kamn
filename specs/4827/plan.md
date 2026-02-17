# Plan — Issue #4827

## Approach

1. Add checker implementation with strict schema validation and deterministic fail-closed behavior.
2. Capture RED first with shell contract test requiring executable checker and expected GO/NO-GO mismatch handling.
3. Add Python unit tests for field resolution, operator evaluation, output generation, and mismatch rejection.
4. Wire new tests into framework regression runner to keep future work on a single entrypoint.

## Affected Modules

- `scripts/framework/declarative_policy_checker.py`
- `scripts/framework/test_declarative_policy_checker.py`
- `scripts/framework/test_declarative_policy_checker_contract.sh`
- `scripts/framework/test_contract_framework.sh`

## Risks / Mitigations

- Risk: policy checks become non-deterministic due to reason ordering drift.
  Mitigation: preserve policy order traversal and assert ordered reason code output in tests.
- Risk: invalid policy payloads slip through and cause runtime ambiguity.
  Mitigation: strict upfront schema validation with explicit failure messages.
- Risk: checker integration regresses existing framework test entrypoint.
  Mitigation: include checker unit + shell contract tests in `test_contract_framework.sh`.

## Interfaces / Contracts

- Input policy schema version: `kamn.framework.declarative-policy.v1`
- Output report schema version: `kamn.framework.declarative-policy-report.v1`
- Stable CLI markers: `status`, `final_decision`, `reason_codes`, `reason_taxonomy_version`, `reason_key`
- Fail-closed mismatch contract: `--expected-final-decision` mismatch exits non-zero with stable mismatch text.

## ADR

No ADR required for this issue. No new dependency, protocol version family, or architecture boundary was introduced.
