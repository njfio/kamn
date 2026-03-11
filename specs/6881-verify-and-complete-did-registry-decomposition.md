# 6881 — Verify and complete did_registry.rs decomposition on current main

## Objective
Verify the exact current-main state of `crates/kamn-core/src/did_registry.rs` and complete the decomposition only if the file is still oversized. If the current baseline is already compliant, record the evidence and close the issue without speculative code churn.

## Inputs/Outputs
### Inputs
- Current `origin/main` implementation at `crates/kamn-core/src/did_registry.rs`
- Existing DID registry extraction contract and real registry behavior tests
- Current AGENTS.md size policy and touched-Rust ratchet

### Outputs
- Verified current-main decomposition state for `did_registry`
- Spec evidence documenting whether any follow-on code change was necessary
- If needed, updated module layout and extraction contract; otherwise, a docs-only closure

## Boundaries / Non-goals
- No changes to DID semantics, registry lifecycle, or chain-submission behavior
- No broad identity redesign outside `did_registry`
- No weakening of existing tests or touched-Rust policy
- No speculative refactor when the verified baseline is already compliant

## Failure modes
- Current baseline is assumed stale when it is already compliant
- Verification closes the issue without exercising the real extraction contract and registry tests
- Evidence omits the mismatch between prior issue history and current verified baseline
- Touched-Rust verification is skipped or run against the wrong repository root

## Acceptance criteria
- [x] Verify the exact current-main state of `crates/kamn-core/src/did_registry.rs`
- [x] If the file is still oversized on current `main`, split it into bounded modules with a thin root shell
- [x] Preserve validation, registry lifecycle, and chain-submission behavior under current tests
- [x] Add or update a hard-fail extraction contract for the intended module layout
- [x] Ensure touched-Rust size policy returns `policy_decision=GO`
- [x] Record any mismatch between prior merged work and current baseline state in the issue spec

## Files to touch
- `specs/6881-verify-and-complete-did-registry-decomposition.md`

## Error semantics
- No runtime behavior changes are allowed in this verification issue
- Verification must fail loudly if the extraction contract or real DID registry tests regress
- No silent closure based on historical issue assumptions

## Test plan
1. Verify the root file size and module tree on current `origin/main`
2. Run the existing extraction contract:
   - `cargo test -p kamn-core --test did_registry_module_extraction_contract -- --nocapture`
3. Run the real DID registry behavior suites:
   - `cargo test -p kamn-core --test did_registry_transactions -- --nocapture`
   - `cargo test -p kamn-core --test did_registry_file_chain_adapter -- --nocapture`
4. Run touched-Rust against the clean verification branch:
   - `TMPDIR=/home/n/Code/kamn/tmp python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6881-clean --base-ref origin/main --output-json /home/n/Code/kamn/tmp/6881-touched-size.json`

## Verified current-main state
- On `origin/main` at verification time, `crates/kamn-core/src/did_registry.rs` is already a thin root shell at `23` LOC
- The extracted module tree already exists under `crates/kamn-core/src/did_registry/`
- The existing extraction contract already exists at `crates/kamn-core/tests/did_registry_module_extraction_contract.rs`
- This means the decomposition is already present on the verified baseline and no source refactor is required in this issue

## Phase 6 evidence
- `cargo test -p kamn-core --test did_registry_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test did_registry_transactions -- --nocapture`
- `cargo test -p kamn-core --test did_registry_file_chain_adapter -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6881-clean --base-ref origin/main --output-json /home/n/Code/kamn/tmp/6881-touched-size.json`
- Touched-Rust result: `policy_decision=GO`

## Deviations
- No new red/green implementation cycle was required because the existing extraction contract and runtime tests already proved the decomposition was present and green on current `main`
- This issue resolves the verification gap between prior merged history and the live baseline rather than landing new source changes
