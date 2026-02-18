# Issue #3880 Plan

- Issue: #3880
- Status: Completed

## Approach
- Add regression tests that lock invalid-profile reason taxonomy for transport profile pair violations.
- Ensure invalid marker-linkage and mixed profile-family paths fail closed with deterministic reason codes.
- Preserve existing production fallback taxonomy coverage to prevent regressions.

## Affected Modules
- crates/kamn-node/src/main_tests/runtime_tests.rs

## Risks and Mitigations
- Risk level: low
- Mitigation: deterministic marker contracts plus drift/regression checks before rollout.

## Interface Contract
- No protocol or wire-format changes without explicit approval and ADR if needed.
- Runtime evidence outputs must remain deterministic and machine-checkable.
- Invalid-profile reason taxonomy remains stable for:
  - `runtime_transport_profile_pair_disallowed`
  - `runtime_transport_profile_fallback_marker_without_in_memory_profile`

## ADR
- No ADR required at planning stage; open ADR if dependency/protocol architecture changes emerge.
