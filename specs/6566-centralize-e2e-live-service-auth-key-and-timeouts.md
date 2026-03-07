# 6566 Centralize E2E Live Service Auth Key and Timeouts

## Objective
Centralize the deterministic service-auth key setup used by the live E2E workflow and require explicit runtime bounds on every live E2E job so the workflow stays deterministic, easier to maintain, and less likely to burn runner time.

## Inputs/Outputs
- Inputs:
  - `.github/workflows/e2e-live.yml`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/e2e_live_workflow_lane.rs`
- Outputs:
  - one centralized service-auth key setup path for the live E2E workflow
  - explicit `timeout-minutes` on `e2e-sdk-direct`, `e2e-mcp-agent`, and `e2e-cli-smoke`
  - fail-closed workflow contract coverage for timeout and centralized-key drift

## Boundaries/Non-goals
- Do not move the deterministic E2E key into GitHub Secrets.
- Do not change live scenario selection, job topology, or runtime behavior beyond centralization and time bounding.
- Do not change production key handling.
- Do not add new dependencies.

## Failure modes
- Workflow keeps three duplicated inline service-auth key setup blocks.
- Any live E2E job lacks explicit `timeout-minutes`.
- Contract tests pass even if the centralized service-auth marker disappears.
- Contract tests pass even if the duplicated inline key setup returns.
- Strategy documentation drifts from the enforced workflow contract.

## Acceptance criteria
- [ ] `.github/workflows/e2e-live.yml` defines the deterministic E2E service-auth key material in one centralized location instead of three duplicated inline copies.
- [ ] `e2e-sdk-direct` declares explicit `timeout-minutes`.
- [ ] `e2e-mcp-agent` declares explicit `timeout-minutes`.
- [ ] `e2e-cli-smoke` declares explicit `timeout-minutes`.
- [ ] `crates/kamn-core/tests/e2e_live_workflow_lane.rs` fails if centralized service-auth markers disappear.
- [ ] `crates/kamn-core/tests/e2e_live_workflow_lane.rs` fails if duplicated inline service-auth key setup returns.
- [ ] `crates/kamn-core/tests/e2e_live_workflow_lane.rs` fails if any E2E live job loses explicit `timeout-minutes`.
- [ ] `docs/ci/strategy.md` remains aligned with the enforced workflow contract markers.

## Files to touch
- `.github/workflows/e2e-live.yml`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/e2e_live_workflow_lane.rs`

## Error semantics
- Workflow contract evaluation remains fail-closed.
- Missing timeout or centralized-key markers must produce deterministic reason codes and a `NO-GO` contract decision.
- Missing strategy markers must continue to fail closed.

## Test plan
- Red:
  - extend `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture` so the current workflow fails on missing timeout and duplicated inline key setup invariants
- Green:
  - update workflow and strategy markers until the contract test passes
- Integration:
  - rerun `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture`

## Deviations
- Centralization went slightly further than the initial minimum slice: the workflow now centralizes both the deterministic private key and its deterministic public key at workflow scope, removing the repeated Python derivation snippet from all three jobs.

## Verification evidence
- `cargo test -p kamn-core --test e2e_live_workflow_lane -- --nocapture`

## Shell-surface closure template
- shell_loc_delta_actual: -3
- rust_loc_delta_actual: 74
- shell_to_rust_ratio_delta_actual: -0.04
- shell_surface_ratio_target_status: improved
