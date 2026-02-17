# Plan — #4408

Status: Reviewed

## Approach

- Extend invariant-fuzz-concurrency summary report payload with deterministic boundary marker fields.
- Extend policy checker reason taxonomy to include boundary drift classes and validate new required fields.
- Keep reason-order canonical by deriving policy reasons from `REASON_CODES_CSV` ordering.
- Update tests and docs together to avoid taxonomy drift.

## Affected Areas

- `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh`
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: required-field expansion may break policy consumers.
  - Mitigation: update all contract tests and emit deterministic fields in pass-path payload.
- Risk: reason ordering nondeterminism.
  - Mitigation: preserve canonical ordering driven by taxonomy CSV constant.
