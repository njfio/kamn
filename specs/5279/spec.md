# Issue #5279 Spec

- Title: Task: implement M9 realtime gateway bridge projection and fail-closed presence-scope contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M9 realtime contracts define deterministic dispatch, anti-spam, backpressure, and scoped-presence behavior, but the gateway boundary does not yet project these outcomes into deterministic transport payload contracts. A bridge layer is needed so runtime gateways consume stable, fail-closed outputs.

## Scope
In:
- Add deterministic gateway projection contracts for:
  - dispatch outcomes (`delivered`/`queued`/backpressure markers),
  - owner-scoped presence connect/query outcomes.
- Add fail-closed branches with stable reason markers for:
  - unsupported gateway transport profile,
  - owner-scope/presence-visibility denials surfaced from M9.
- Add gateway-bridge tests validating deterministic projection and fail-closed branches.

Out:
- Live websocket fan-out/session orchestration in `kamn-node`.
- External push transport integrations.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 360
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: M9 dispatch outcomes project deterministic gateway event envelopes for supported transports.
- AC-2: M9 presence connect/query outcomes project deterministic owner-scoped presence envelopes.
- AC-3: Unsupported transport and scope/visibility denial branches fail closed with stable reason markers.
- AC-4: Unit/Functional/Integration/Regression coverage for this slice passes with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid dispatch request + supported transport + admissible channel/anti-spam state | deterministic gateway dispatch envelope fields (event/ack/queue/backpressure/reason) |
| C-02 | AC-2 | Functional | valid presence connect + owner-scoped self query + supported transport | deterministic gateway presence envelope with visible state |
| C-03 | AC-3 | Regression | unsupported transport profile | fail-closed bridge error with unsupported-transport reason code |
| C-04 | AC-3 | Regression | cross-owner presence query after connect | fail-closed bridge error preserving owner-scope denial reason marker |
| C-05 | AC-3 | Regression | same-owner presence query without linkage | fail-closed bridge error preserving presence-visibility denial reason marker |
| C-06 | AC-4 | Verification | fmt/clippy + targeted bridge tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_gateway_bridge`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- Story `#5252` begins Phase-5 integration with deterministic gateway bridge contracts over M9 domain logic.
- No shell-surface growth while increasing runtime-integration-ready Rust contract coverage.
