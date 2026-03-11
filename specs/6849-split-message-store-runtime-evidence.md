# Objective
Split `build_data_layer_runtime_evidence()` in `crates/kamn-node/src/service_api_endpoint/message_store.rs` into a small coordinator plus bounded helpers while preserving the existing M0-M11 evidence-building behavior for the service API message-store path.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-node/src/service_api_endpoint/message_store.rs`
  - existing service API message persistence flow that calls `build_data_layer_runtime_evidence()`
  - current sender/recipient DID normalization and M0-M11 evidence generation behavior
- Outputs:
  - a small `build_data_layer_runtime_evidence()` coordinator
  - bounded helper functions or sibling module(s) for normalized identities, envelope/ledger assembly, evidence stage execution, and final record mapping
  - regression and extraction tests that prove behavior remains equivalent and the touched surface stays within the active size policy

## Boundaries/Non-goals
- Do not change the `ServiceApiDataLayerRuntimeEvidenceRecord` schema.
- Do not change public API contracts or service API endpoint behavior.
- Do not rewrite unrelated message-store flows.
- Do not introduce new dependencies.

## Failure modes
- Sender/recipient DID normalization changes and breaks deterministic fallback behavior.
- Any M0-M11 stage loses fail-closed error propagation or changes its error marker.
- The extracted coordinator or helpers still exceed the active 25-line function limit.
- The persisted runtime evidence payload changes semantics under the existing integration path.

## Acceptance criteria
- [x] `build_data_layer_runtime_evidence()` is reduced below the 25-line function limit via coordinator/helper extraction.
- [x] Evidence-building behavior remains equivalent under regression coverage for the real send-path integration.
- [x] Success and failure-path tests cover the extracted seams.
- [x] No newly introduced helper exceeds the function-size limit.
- [x] Error semantics remain fail-closed and observable with deterministic stage markers.

## Files to touch
- `specs/6849-split-message-store-runtime-evidence.md`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/tests/message_store_runtime_evidence_extraction_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_runtime_evidence_contract_tests.rs`
- optional sibling support files if the extraction needs a bounded module seam

## Error semantics
- Each extracted stage must keep returning `Result<_, String>` with deterministic fail-closed markers.
- No stage may swallow an underlying error or silently substitute a fallback record.
- DID normalization must remain explicit and deterministic, including the alternate recipient fallback when sender and recipient normalize to the same value.

## Test plan
1. Add a red extraction contract that fails while `build_data_layer_runtime_evidence()` exceeds the function-size limit.
2. Add or extend regression coverage for the real send-path evidence integration and for deterministic failure markers on extracted seams.
3. Refactor the evidence builder into bounded helpers.
4. Re-run the extraction contract and targeted `kamn-node` evidence tests until green.
5. Run the touched-Rust size ratchet on the final write set.

## Final evidence
- `cargo test -p kamn-node --test message_store_runtime_evidence_extraction_contract -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11 -- --nocapture`
- `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6849-touched-size.json`

## Deviations
- None.
