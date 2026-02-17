# Plan — #4352

Status: Reviewed

## Approach

- Add ratio fields to rustdoc artifact lane report generator (`kamn_core_rustdoc_artifact_contract_lane_impl.sh`).
- Validate those fields in `check_kamn_core_rustdoc_artifact_policy.sh` with deterministic fail reason.
- Keep existing schema and pass/fail semantics compatible via additive fields and explicit reason key.
