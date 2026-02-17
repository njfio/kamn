# Plan — #4402

Status: Reviewed

## Approach

- Extend invariant-fuzz-concurrency policy tests with RED tamper cases for fuzz seed replay count drift, concurrency race misclassification, and CI/local boundary marker drift.
- Add CI/local boundary marker fields to invariant-fuzz-concurrency summary reports.
- Harden policy checker with deterministic reason taxonomy coverage for boundary drift and replay regression classes.
- Keep normalized reason outputs stable (`invariant_policy_reason_codes_value`, expected/observed reason mapping).
- Update CI strategy documentation with new boundary marker/reason contracts.

## Affected Areas

- `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh`
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: taxonomy updates drift between lane summary, checker, and tests.
  - Mitigation: derive order from single CSV constant and assert in both policy + lane tests.
- Risk: boundary marker changes break existing downstream parsers.
  - Mitigation: additive field updates with deterministic marker names and doc parity in same PR.
