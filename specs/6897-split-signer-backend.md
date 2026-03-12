# 6897-split-signer-backend

## Objective
Split `crates/kamn-core/src/signer_backend.rs` into bounded concern-based modules while preserving current request validation, secure-provider handshake policy, backend signing/verification behavior, routing, and error semantics.

## Inputs/Outputs
- Input: current `signer_backend.rs` production logic and inline tests
- Output: a thin root module plus bounded sibling modules for request/env helpers, provider and role policy, backend implementations, router logic, error types, and tests
- Output: a hard-fail extraction contract that enforces the new layout

## Boundaries/Non-goals
- Do not change signer wire formats, signature preimage semantics, or handshake policy behavior
- Do not redesign the signer backend public API beyond what the split requires
- Do not add new signer providers, routing modes, or crypto dependencies

## Failure modes
- Extraction contract does not fail while the root file remains oversized or expected modules are missing
- Request validation or env-resolution behavior drifts during extraction
- Secure provider handshake policy or key-role segregation changes silently
- Local or secure backend signing/verification behavior changes
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [ ] `crates/kamn-core/src/signer_backend.rs` becomes a thin root shell under the active file-size policy
- [ ] concern-based sibling modules are introduced for request/env helpers, provider-role policy, backend implementations, router logic, error surface, and tests
- [ ] no touched extracted file exceeds the active touched-Rust size policy
- [ ] a hard-fail extraction contract exists and passes
- [ ] existing signer backend tests still pass
- [ ] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-core/src/signer_backend/`
- `crates/kamn-core/tests/signer_backend_module_extraction_contract.rs`
- `specs/6897-split-signer-backend.md`

## Error semantics
- Preserve existing `SignerBackendError` variants and message text unless extraction only clarifies implementation structure without changing meaning
- No silent fallback paths may be introduced during extraction
- Validation and provider-policy failures remain fail-closed at the function boundary

## Test plan
- Add a red extraction contract that fails while the root file is still oversized and the expected module layout is missing
- Run the extraction contract target
- Run the existing signer backend tests after the split
- Run touched-Rust size policy on the final branch

## Phase 6 evidence
- `cargo test -p kamn-core --test signer_backend_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core signer_backend::tests:: --lib -- --nocapture`
- `cargo test -p kamn-node --no-run`
- `cargo test -p kamn-sdk --no-run`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6897-touched-size-post-refactor.json`
- Result: `policy_decision=GO`

## Deviations
- Phase 6 integration proof used downstream `kamn-node` and `kamn-sdk` compile verification with `--no-run` rather than a new behavior test because this change is an internal `kamn-core` module split with no new runtime entrypoint.
