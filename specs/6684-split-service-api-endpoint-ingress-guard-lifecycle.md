# Issue 6684: Split service_api_endpoint_tests Ingress Guard Lifecycle Tranche

## Objective

Extract the ingress guard coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file keeps shrinking while the rate-limit, anti-spam, replay, concurrency, and lifecycle projection paths remain exercised from the real `kamn-node` test entrypoint.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- a new ingress-guard-lifecycle submodule declaration from the root test file
- moved rate-limit, anti-spam, replay, concurrency, and lifecycle projection coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not change service API runtime behavior or reason-code semantics
- Do not extract unrelated route/rendering, persistence, bridge, or mailbox coverage in this issue
- Do not weaken ingress-guard assertions to make the split pass
- Keep the tranche limited to ingress budget, anti-spam, replay, concurrency, and lifecycle projection coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps the moved ingress-guard test markers after extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real ingress guard, replay, or lifecycle projection path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new ingress-guard-lifecycle submodule and no longer retains the moved ingress-guard test markers.
- [x] AC-2: Extracted ingress-guard-lifecycle files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `4525` to `2400` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted ingress-guard integration test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6684-split-service-api-endpoint-ingress-guard-lifecycle.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains the moved ingress-guard coverage or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` ingress guard and lifecycle projection paths.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the ingress-guard-lifecycle module markers, moved test markers, and a lowered staged root threshold.
2. Extract the ingress guard coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` ingress-guard tests.
4. Record integration evidence and any deviations in this spec.

## Deviations

- No behavioral deviations were introduced. The extraction preserved the existing ingress guard assertions and kept the moved coverage on the real `kamn-node` test path.

## Phase 6 Evidence

- Root wiring:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` declares `#[path = "service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs"]` and `mod ingress_guard_lifecycle_contract_tests;`
- File sizes:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `1973`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs`: `12`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/concurrency_guard_contract_tests.rs`: `149`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/ingress_budget_contract_tests.rs`: `85`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/lifecycle_projection_contract_tests.rs`: `127`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/replay_guard_contract_tests.rs`: `95`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/sender_anti_spam_contract_tests.rs`: `59`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/support.rs`: `134`
- Touched-Rust size policy:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6684-touched-size.json`
  - Result: `status=pass`, `policy_decision=GO`
- Targeted evidence:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded -- --nocapture`
