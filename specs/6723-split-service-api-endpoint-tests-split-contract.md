# Objective

Split `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` into bounded sibling modules that preserve the current extraction guarantees for the `service_api_endpoint_tests` tree while reducing the root contract shell under the active size policy.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` at 1500 LOC on current `origin/main`
- Existing service API endpoint test modules already organized by concern under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- Current `kamn-node` test entrypoint wiring for `service_api_endpoint_tests_split_contract`
- Active touched-Rust size policy

## Outputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` reduced to a bounded root shell
- New bounded sibling modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract/` that mirror the current service API endpoint concern seams
- Contract coverage that still enforces root-shell budget, module presence, and moved-marker ownership
- Updated issue/spec evidence showing the extracted contract surface still runs through the real `kamn-node` test entrypoint and passes touched-Rust size checks

# Boundaries/Non-goals

- Do not change service API endpoint runtime behavior
- Do not change the semantics of the existing service API endpoint tests
- Do not redesign unrelated `kamn-node` main test modules
- Do not weaken or remove moved-marker ownership checks to make the file smaller

# Failure modes

- `service_api_endpoint_tests_split_contract.rs` remains an oversized monolith
- extracted contract modules are arbitrary slices rather than matching the existing service API endpoint concern seams
- moved-marker ownership checks are lost during extraction
- the root contract no longer enforces the service API endpoint root-shell budget or extracted module layout
- touched-Rust size policy fails on the issue write set

# Acceptance criteria (testable booleans)

- [ ] `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` is reduced to a bounded root shell under the active size policy
- [ ] extraction-contract checks are split into coherent sibling modules that mirror the existing service API endpoint concern seams
- [ ] the extracted contract files added by this issue remain within the active touched-Rust size policy on the issue write set
- [ ] the contract root still enforces root-shell budget, module-layout presence, and moved-marker ownership across the service API endpoint test tree
- [ ] `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes after extraction
- [ ] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6723-split-service-api-endpoint-tests-split-contract.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- new files under `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract/`
- `crates/kamn-node/tests/service_api_endpoint_tests_split_contract_extraction_contract.rs`

# Error semantics

- Preserve current hard-fail assertions for missing extracted module files, unexpected retained markers, and root-shell budget regressions
- New helper modules must fail with the same exact missing-path and ownership context the existing contract provides
- No fallback path to inline monolith checks or partial contract execution

# Test plan

1. Add a red contract that requires a bounded root shell and a concern-based `service_api_endpoint_tests_split_contract/` module layout.
2. Extract the current contract checks into sibling modules that mirror the existing service API endpoint test seams.
3. Run `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`.
4. Run `cargo test -p kamn-node --test service_api_endpoint_tests_split_contract_extraction_contract -- --nocapture`.
5. Run the touched-Rust size policy on the issue write set.
6. Record final evidence and any deviations in this spec before opening the PR.

# Phase 6 evidence

- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` reduced from `1500` LOC to `33` LOC as a thin root shell.
- Extracted split-contract files are all within budget:
  - `auth_scope_contract_tests.rs`: `102` LOC
  - `bridge_persistence_restart_contract_tests.rs`: `52` LOC
  - `budget_contract_tests.rs`: `36` LOC
  - `channel_agent_directory_contract_tests.rs`: `82` LOC
  - `content_lifecycle_restart_contract_tests.rs`: `53` LOC
  - `ingress_guard_lifecycle_contract_tests.rs`: `169` LOC
  - `mailbox_relay_delivery_contract_tests.rs`: `154` LOC
  - `message_persistence_contract_tests.rs`: `82` LOC
  - `residual_root_contract_tests.rs`: `133` LOC
  - `root_layout_contract_tests.rs`: `48` LOC
  - `route_render_contract_tests.rs`: `71` LOC
  - `shared_support_contract_tests.rs`: `177` LOC
  - `support.rs`: `131` LOC
  - `task_escrow_persistence_contract_tests.rs`: `74` LOC
  - `transport_surface_observability_contract_tests.rs`: `140` LOC
  - `websocket_contract_tests.rs`: `62` LOC
- Real `kamn-node` split-contract target passed after extraction:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
- External extraction contract passed after extraction:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node --test service_api_endpoint_tests_split_contract_extraction_contract -- --nocapture`
- Touched-Rust ratchet passed:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6723-touched-size.json`
  - result: `status=pass`, `policy_decision=GO`

# Deviations

- `#6723` was rebuilt on a fresh post-`#6724` branch because current `main` had an unrelated observability-endpoint import regression that blocked `kamn-node` compilation during the first attempt. `#6724` restored the baseline first, then this issue was rebuilt cleanly on top of the repaired `main`.
