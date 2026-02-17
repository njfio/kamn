# Plan — #4406

Status: Reviewed

## Approach

- Refactor `check_invariant_fuzz_concurrency_policy.sh` into deterministic validation phases:
  - schema/field validation;
  - replay metadata validation;
  - lane/runtime expected-reason derivation;
  - status/reason contract conformance check.
- Emit deterministic policy evidence markers:
  - taxonomy version;
  - supported reason-code CSV;
  - expected/observed reason values;
  - fail-closed final decision.
- Extend combined lane summary payload with stable taxonomy markers to prevent evidence drift.
- Update docs with invariant reason-taxonomy requirements for go/no-go governance.

## Affected Areas

- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/testing/invariant-and-fuzz-strategy.md`

## Risks and Mitigations

- Risk: fail-path marker emissions differ from existing checker output.
  - Mitigation: preserve existing success markers and add explicit deterministic fail markers used by tests.
- Risk: reason ordering instability causes flaky tests.
  - Mitigation: canonical, fixed-order reason code definitions.

## Contract Notes

- Checker must remain fail-closed on any contract mismatch.
- New marker fields are additive and deterministic.
