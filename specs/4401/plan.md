# Plan — #4401

Status: Reviewed

## Approach

- Extend invariant policy checker tests with explicit red cases for:
  - lane-failure acceptance drift;
  - taxonomy/evidence output drift.
- Introduce deterministic reason-mapping logic in `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh` that:
  - computes expected lane/runtime-derived reason codes;
  - validates observed `status` and `reason_codes` against expected values;
  - emits stable policy markers for pass/fail paths.
- Normalize lane summary evidence in `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh` with deterministic taxonomy marker fields.
- Update release checklist and strategy docs with invariant policy taxonomy markers and mapping contract.

## Affected Areas

- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/testing/invariant-and-fuzz-strategy.md`

## Risks and Mitigations

- Risk: changing checker output breaks existing call sites.
  - Mitigation: preserve existing `status=ok` + `final_decision=GO` pass markers; add markers additively.
- Risk: reason mapping order instability.
  - Mitigation: fixed canonical reason-code ordering and deterministic CSV formatting.

## Contract Notes

- Keep fail-closed behavior (non-zero exit on policy mismatch).
- Keep bounded CI-smoke runtime behavior and existing lane wrappers/manifests unchanged.
