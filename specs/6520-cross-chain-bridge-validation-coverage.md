## Objective
Add explicit coverage for the `UnauthorizedListener` and `RouteTargetMismatch` inbound-validation
branches in `CrossChainBridgeEngine::validate_inbound_request()`.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/cross_chain_bridge.rs`
  - `crates/kamn-core/tests/cross_chain_bridge.rs`
- Outputs:
  - one focused test for `UnauthorizedListener`
  - one focused test for `RouteTargetMismatch`
  - preserved existing inbound success and unknown-route behavior

## Boundaries/Non-goals
- Do not change production bridge behavior unless red-phase investigation proves a branch is
  unreachable or miswired.
- Do not add new modules or dependencies.
- Do not modify CI/workflow surfaces.

## Failure modes
- `UnauthorizedListener` exists in production code but is not directly exercised by tests.
- `RouteTargetMismatch` exists in production code but is not directly exercised by tests.
- Adding the new coverage weakens or replaces nearby inbound bridge assertions instead of pinning
  the branch outcomes directly.

## Acceptance criteria
- [ ] `crates/kamn-core/tests/cross_chain_bridge.rs` contains a dedicated test for
      `UnauthorizedListener`.
- [ ] `crates/kamn-core/tests/cross_chain_bridge.rs` contains a dedicated test for
      `RouteTargetMismatch`.
- [ ] Existing nearby bridge tests remain green.
- [ ] Targeted cross-chain bridge tests pass locally.

## Files to touch
- `crates/kamn-core/tests/cross_chain_bridge.rs`
- `specs/6520-cross-chain-bridge-validation-coverage.md`

## Error semantics
- Inbound validation must remain fail-closed.
- Unauthorized listeners must surface `UnauthorizedListener`.
- Target DID mismatches must surface `RouteTargetMismatch`.
- No fallback or normalization is allowed.

## Test plan
- Red:
  - add the two focused inbound-validation tests
  - confirm they fail before any production change
- Green:
  - `cargo test -p kamn-core --test cross_chain_bridge -- --nocapture`
- Refactor:
  - rerun the same targeted bridge suite after cleanup

## Deviations
- Pending red-phase investigation may show the production behavior already exists and only the
  missing coverage needs to land.

## Execution Evidence
- Pending.
