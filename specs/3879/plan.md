# Issue #3879 Plan

- Issue: #3879
- Status: Completed

## Approach
- Add runtime transport profile compatibility checks for unsupported live/fallback pairings.
- Add deterministic reason-code classification for fallback marker linkage violations.
- Preserve existing production fallback rejection behavior and document the expanded taxonomy.

## Affected Modules
- crates/kamn-node/src/runtime_orchestration.rs
- crates/kamn-node/src/main_tests/runtime_tests.rs
- docs/foundation/runtime-network.md
- docs/architecture/p2p-transport.md
- crates/kamn-core/tests/runtime_network_docs.rs

## Risks and Mitigations
- Risk level: low
- Mitigation: deterministic marker contracts plus drift/regression checks before rollout.

## Interface Contract
- No protocol or wire-format changes without explicit approval and ADR if needed.
- Runtime evidence outputs must remain deterministic and machine-checkable.
- Added deterministic compatibility reason codes:
  - `runtime_transport_profile_pair_disallowed`
  - `runtime_transport_profile_fallback_marker_without_in_memory_profile`

## ADR
- No ADR required at planning stage; open ADR if dependency/protocol architecture changes emerge.
