# Plan — #4407

Status: Reviewed

## Approach

- Extend `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh` with targeted tamper cases for fuzz replay count drift and concurrency misclassification.
- Add contract-lane marker assertions in `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh` for new boundary fields.
- Capture RED failure before checker changes.

## Affected Areas

- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`

## Risks and Mitigations

- Risk: red fixtures fail for unrelated fields.
  - Mitigation: mutate one contract field at a time from known-good report fixture.
