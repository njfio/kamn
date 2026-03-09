# 6638-add-runtime-wiring-transport-profile-coverage

## Objective
Add dedicated crate-level contract and integration coverage for the public runtime wiring and
transport-profile API in `runtime_peer_coordination.rs` so marker selection, disabled-gossip
behavior, role-specific wiring, and compile-mode alignment stay pinned outside broader live
transport tests.

## Inputs/Outputs
- Inputs:
  - public `RuntimeWiring::all_components()`
  - public `RuntimeTransportProfile`
  - public `Libp2pCompileMode`
  - public `libp2p_feature_gate_name()`
  - public `resolve_libp2p_compile_mode()`
  - public `build_runtime_wiring_with_transport_profile(...)`
  - public `build_runtime_wiring(...)`
- Outputs:
  - dedicated integration test surface at
    `crates/kamn-core/tests/runtime_wiring_transport_profile_integration.rs`
  - dedicated contract test at
    `crates/kamn-core/tests/runtime_wiring_transport_profile_contract.rs`
  - refreshed `test_file_size_policy` baseline if test inventory count changes

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/runtime_peer_coordination.rs`
- No authenticated peer frame, proposal planner, or runtime-network-fault coverage in this issue
- No CI or workflow changes
- No visibility expansion for internal helpers just to support tests

## Failure modes
- dedicated runtime-wiring integration surface missing entirely
- default runtime wiring stops emitting deterministic in-memory transport markers
- live libp2p transport wiring stops emitting transport profile, provider, or compile-mode markers
- disabled gossip stops fail-closing to deterministic `gossip-transport-disabled` marker behavior
- role-specific wiring stops returning deterministic processor/listener/approver components
- feature gate name or compile-mode marker alignment drifts
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [x] `runtime_wiring_transport_profile_contract` fails when the dedicated integration surface or
      its required marker cases disappear
- [x] integration coverage asserts default `build_runtime_wiring(...)` returns in-memory transport
      profile markers and excludes live-provider markers
- [x] integration coverage asserts
      `build_runtime_wiring_with_transport_profile(..., RuntimeTransportProfile::Libp2pLive)`
      emits live transport, provider, and compile-mode markers through the public API
- [x] integration coverage asserts gossip-disabled wiring emits deterministic disabled-gossip marker
      and excludes transport markers
- [x] integration coverage asserts processor/listener/approver configs return their expected
      public role components
- [x] integration coverage asserts `libp2p_feature_gate_name()` and
      `resolve_libp2p_compile_mode().marker_component()` stay aligned with the active feature state
- [x] `cargo test -p kamn-core --test runtime_wiring_transport_profile_contract -- --nocapture`
      passes
- [x] `cargo test -p kamn-core --test runtime_wiring_transport_profile_integration -- --nocapture`
      passes
- [x] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6638-add-runtime-wiring-transport-profile-coverage.md`
- `crates/kamn-core/tests/runtime_wiring_transport_profile_contract.rs`
- `crates/kamn-core/tests/runtime_wiring_transport_profile_integration.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if inventory count changes

## Error semantics
- Tests assert the existing public behavior only
- No new production error types or translation layers are introduced
- Feature-gate and wiring marker assertions must use the existing public API surface

## Test plan
1. Add a contract test that references
   `runtime_wiring_transport_profile_integration.rs` before that file exists so RED is a real
   missing-surface failure.
2. Add dedicated integration coverage for default in-memory wiring, live libp2p wiring,
   gossip-disabled wiring, role-specific components, and compile-mode marker alignment.
3. Run targeted contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change
   inventory counts.

## Deviations
- The workspace `test_file_size_policy` inventory changed from `489` to `491`, so
  `fixtures/ci/test_file_size_policy_baseline.env` was refreshed during integration.

## Phase 6 Evidence
- `cargo test -p kamn-core --test runtime_wiring_transport_profile_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_wiring_transport_profile_integration -- --nocapture`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
