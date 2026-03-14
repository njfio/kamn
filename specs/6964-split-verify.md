# 6964-split-verify

## Objective
Reduce `crates/kamn-e2e-harness/src/verify.rs` to a thin root shell by extracting deterministic verification responsibilities into bounded concern-based modules without changing verification semantics or CLI-facing behavior.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-e2e-harness/src/verify.rs`
  - existing `kamn-e2e-harness` verification tests and callers
- Outputs:
  - thin root `verify.rs` shell
  - bounded modules for manifest validation, chain-dump parsing, evidence verification, report generation, and tests
  - hard-fail extraction contract guarding the split
  - updated spec evidence for the decomposition

## Boundaries/Non-goals
- Do not change verification semantics, output formats, or public API signatures unless required to preserve compilation under the split.
- Do not add dependencies.
- Do not do broad warning cleanup outside the touched surface.
- Do not rewrite evidence logic beyond what is needed to split and keep behavior identical.

## Failure modes
- `verify.rs` remains over the root file budget.
- Extracted modules exceed the file budget.
- Deterministic report JSON ordering changes.
- Chain-dump verification stops enforcing genesis/hash continuity.
- Evidence verification stops rejecting malformed `_verification` fields.
- Inline tests are lost or no longer compile.

## Acceptance criteria
- [x] `crates/kamn-e2e-harness/src/verify.rs` is reduced to a thin root shell within the active file budget.
- [x] Extraction contract fails closed if root markers or module files regress.
- [x] `verify_manifest`, `verify_chain_dump`, `validate_evidence_verification_blocks`, `generate_verification_report`, and `generate_verification_report_json` remain callable from the root.
- [x] Existing verification unit tests still pass.
- [x] Touched-Rust size policy returns `policy_decision=GO`.

## Files to touch
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/src/verify/*.rs`
- `crates/kamn-e2e-harness/tests/verify_module_extraction_contract.rs`
- `specs/6964-split-verify.md`

## Error semantics
- Preserve current `Result<_, String>` error behavior at the root API.
- Keep hard-fail validation behavior for missing markers and malformed verification fields.
- Do not add silent fallbacks or soften any existing validation path.

## Test plan
- Red extraction contract asserting root budget, root markers, and extracted module presence.
- Existing verification unit tests from `verify.rs`.
- `cargo test -p kamn-e2e-harness --test verify_module_extraction_contract -- --nocapture`.
- `cargo test -p kamn-e2e-harness verify::tests:: --lib -- --nocapture`.
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6964-touched-size.json`.

## Final evidence
- Extracted modules landed under `crates/kamn-e2e-harness/src/verify/`:
  - `manifest.rs`
  - `chain_dump.rs`
  - `evidence.rs`
  - `report.rs`
  - `support.rs`
  - `tests.rs`
- Root shell reduced to `19` LOC in `crates/kamn-e2e-harness/src/verify.rs`.
- Verified:
  - `cargo test -p kamn-e2e-harness --test verify_module_extraction_contract -- --nocapture`
  - `cargo test -p kamn-e2e-harness verify::tests:: --lib -- --nocapture`
  - `cargo test -p kamn-e2e-harness --test command_contract parser_verify_contract_tests:: -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6964-touched-size.json`

## Deviations
- None.
