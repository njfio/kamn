# Plan — #4344

Status: Reviewed

## Approach

- Extend rustdoc artifact contract report schema with docs/behavioral count + ratio fields.
- Extend rustdoc artifact policy checker to validate ratio fields and fail closed with deterministic reason markers.
- Add RED assertions in policy checker tests for ratio imbalance failure.
- Update CI/runtime docs to publish ratio governance markers.

## Affected Areas

- `scripts/ci/kamn_core_rustdoc_artifact_contract_lane_impl.sh`
- `scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh`
- `scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
- `docs/ci/strategy.md`
- `docs/architecture/runtime.md`

## Risks and Mitigations

- Risk: schema drift between lane report and policy checker.
  - Mitigation: add contract test assertions for both pass and fail ratio paths.
- Risk: ratio policy over-tightening.
  - Mitigation: deterministic threshold field in report + explicit max-ratio validation.
