# 6946-split-signer-root

## Objective
Reduce `crates/kamn-node/src/signer.rs` to a thin root shell under the active file-size policy by extracting concern-based modules for signer models, secret/env validation, managed signing flow, direct payload rendering, and tests, without changing signer CLI or backend behavior.

## Inputs/Outputs
- Inputs:
  - Current `crates/kamn-node/src/signer.rs`
  - Existing adjacent modules: `managed_backend`, `nonce`, `signer_adapter`, `signer_policy`
  - Existing signer tests and node compile surfaces
- Outputs:
  - Thin root `signer.rs`
  - Bounded sibling modules under `crates/kamn-node/src/signer/`
  - Hard-fail extraction contract covering root shell budget and module layout
  - Updated spec evidence and PR linking back to `#6946`

## Boundaries/Non-goals
- Do not change signer command semantics or public behavior.
- Do not introduce new signer backends.
- Do not weaken signer validation, provenance checks, or error handling.
- Do not refactor adjacent modules beyond what is required for the root split.

## Failure modes
- Extraction contract does not fail red on the oversized root.
- Root shell still exceeds active size budget after extraction.
- Managed-external signing path regresses.
- Strict signer secret-source precedence checks regress.
- Touched-Rust policy fails because extracted files/functions remain oversized.

## Acceptance criteria
- [ ] `crates/kamn-node/src/signer.rs` is reduced to a thin root shell under the active file-size policy.
- [ ] Concern-based modules are extracted for signer models, env/secret handling, managed signing flow, direct payload rendering, and tests.
- [ ] A hard-fail extraction contract exists and passes on the final write set.
- [ ] Existing signer adapter boundary tests remain green.
- [ ] Existing managed-backend regressions remain green.
- [ ] Touched-Rust returns `policy_decision=GO` on the final write set.

## Files to touch
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/*.rs`
- `crates/kamn-node/tests/signer_module_extraction_contract.rs`
- `specs/6946-split-signer-root.md`

## Error semantics
- Preserve existing `ConfigError::RuntimeKolmeLive` and related signer error behavior exactly.
- Validation remains fail-closed and eager.
- No silent fallbacks or error swallowing.

## Test plan
- Red extraction contract for root shell budget and module layout.
- Green extraction contract after split.
- Real signer regression tests:
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract -- --nocapture`
  - `cargo test -p kamn-node regression_managed_external_backend_scrubs_signer_secret_env_for_child_process -- --nocapture`
  - `cargo test -p kamn-node regression_managed_external_backend_rejects_unterminated_quote_in_command_spec -- --nocapture`
- Touched-Rust validation with the Python entrypoint on the final write set.
