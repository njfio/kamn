# 6915-split-managed-backend

## Objective
Split `crates/kamn-node/src/signer/managed_backend.rs` into bounded concern-based modules while preserving managed-external signer behavior for env/config validation, command parsing, child-process execution, public-key normalization, signature provenance verification, adapter wiring, and existing tests.

## Inputs/Outputs
- Input: current `crates/kamn-node/src/signer/managed_backend.rs` production source and its existing tests.
- Output: a thin root shell delegating to bounded sibling modules for command/env resolution, backend response parsing and provenance checks, child-process execution, key-marker normalization, adapter/signing entrypoints, and tests.
- Output: a hard-fail extraction contract that enforces the root shell budget and extracted module layout.

## Boundaries/Non-goals
- Do not change signer semantics, reason markers, or public API behavior.
- Do not change managed-external command environment names, timeout defaults, or handshake semantics.
- Do not add dependencies.
- Do not weaken or delete existing tests to make the split pass.

## Failure modes
- The root file remains oversized while the extraction contract passes.
- Command parsing or env validation semantics drift during extraction.
- Signature provenance verification changes reason markers or validation behavior.
- Child-process execution loses fail-closed behavior for timeout, stderr, or malformed output paths.
- Any touched extracted file or function fails the touched-Rust size policy.

## Acceptance criteria
- [ ] `crates/kamn-node/src/signer/managed_backend.rs` becomes a thin root shell under the active file-size policy.
- [ ] Bounded sibling modules exist for command/env resolution, response parsing/provenance verification, child-process execution, key-marker normalization, adapter/signing entrypoints, and tests.
- [ ] Existing managed-backend behavior remains unchanged after the split.
- [ ] A hard-fail extraction contract exists and passes.
- [ ] The real managed-backend target(s) still pass after the split.
- [ ] Touched-Rust size policy returns `GO` on the final branch.

## Files to touch
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/src/signer/managed_backend/`
- `crates/kamn-node/tests/managed_backend_module_extraction_contract.rs`
- `specs/6915-split-managed-backend.md`

## Error semantics
- Existing fail-closed `ConfigError::RuntimeKolmeLive(...)` behavior remains externally observable and deterministic.
- No new fallbacks, silent coercions, or swallowed command/env/provenance failures are introduced.
- Malformed command specs, malformed backend output, invalid public keys, and execution timeouts remain explicit hard failures.

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing.
- Run the extraction contract target and confirm red.
- Run the real managed-backend target after extraction.
- Run touched-Rust size policy on the final branch.
