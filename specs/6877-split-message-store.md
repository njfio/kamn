## Objective

Split `crates/kamn-node/src/service_api_endpoint/message_store.rs` into bounded concern-based modules so the service API message-store surface is reviewable and stays under the active size policy without changing runtime behavior.

## Inputs/Outputs

Inputs:
- Current `ServiceApiMessageStore` runtime behavior in `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- Existing service API message-store tests in `crates/kamn-node/src/service_api_endpoint/tests.rs`
- Existing runtime-evidence extraction contract in `crates/kamn-node/tests/message_store_runtime_evidence_extraction_contract.rs`

Outputs:
- A thin `message_store.rs` root that delegates to bounded sibling modules
- Extracted modules for persisted state/types, persistence/state-file IO, store operations, and runtime-evidence helpers
- A new extraction contract that hard-fails if the root regresses into a monolith
- Existing real-path message-store and service API tests still green

## Boundaries/Non-goals

Non-goals:
- Changing service API message semantics
- Changing runtime-evidence field values or response contracts
- Redesigning service API routes outside the message-store surface
- Modifying unrelated middleware or server wiring

Boundaries:
- Keep changes scoped to `crates/kamn-node/src/service_api_endpoint/message_store*` and directly related tests/specs
- Preserve current typed/structured error strings and state-file semantics
- Reuse the existing runtime-evidence helper seams rather than reworking their algorithms

## Failure modes

- Message/task/content/bridge persistence behavior changes during extraction
- State-file refresh/persist ordering regresses and loses idempotency guarantees
- Agent registration conflict/persistence behavior drifts
- Runtime-evidence generation stops matching the existing integration test expectations
- The root file remains oversized or re-accumulates moved sections after extraction

## Acceptance criteria

- [ ] `crates/kamn-node/src/service_api_endpoint/message_store.rs` is reduced to a thin root shell within the staged file-size target for this issue
- [ ] Persisted state/type definitions live in bounded sibling modules instead of the root file
- [ ] State-file persistence/refresh logic and store operation methods are extracted into bounded sibling modules
- [ ] Runtime-evidence helpers remain bounded and continue to satisfy the existing runtime-evidence extraction contract
- [ ] A new extraction contract fails if the root regains moved state/store sections
- [ ] `cargo test -p kamn-node --test message_store_runtime_evidence_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-node service_api_endpoint::tests:: -- --nocapture` passes
- [ ] touched-Rust size policy passes on the write set

## Files to touch

- `specs/6877-split-message-store.md`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/`
- `crates/kamn-node/tests/message_store_module_extraction_contract.rs`
- `crates/kamn-node/tests/message_store_runtime_evidence_extraction_contract.rs` (only if marker expectations need extension)

## Error semantics

- Extraction must preserve current hard-fail `Result<_, String>` behavior and path-specific parse/persist error messages
- No silent fallback may be introduced in persistence, refresh, registration, or runtime-evidence paths
- Extraction contract failures must name the missing module markers or oversized root sections directly

## Test plan

Red:
- Add `message_store_module_extraction_contract.rs` asserting the new module layout and staged root budget; it must fail on current `main`
- Re-run the existing runtime-evidence extraction contract and service API message-store tests to confirm the root is still monolithic before the split

Green:
- Extract state/types, persistence/store operations, and runtime-evidence seams into bounded sibling modules
- Keep the root as imports/re-exports plus minimal orchestration only

Refactor/Integration:
- Run the message-store extraction contract
- Run the existing runtime-evidence extraction contract
- Run the real message-store/service API tests exercising persistence and runtime evidence
- Run touched-Rust size policy on the full write set

## Phase 6 evidence

- `cargo test -p kamn-node --test message_store_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-node --test message_store_runtime_evidence_extraction_contract -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11 -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6877-remote-clean-1773246311 --base-ref origin/main --output-json /tmp/6877-remote-clean-size.json`
- touched-Rust result: `policy_decision=GO`

## Deviations

- Clean-clone verification used the Python touched-Rust entrypoint directly to keep repo-root resolution exact for the standalone clone.
