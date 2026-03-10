# Objective

Extract the oversized release-series docs-contract surface from `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs` into bounded sibling modules while preserving the real `docs_contract_release_group` test target and all current docs-marker and milestone-index assertions.

# Inputs/Outputs

Inputs:
- `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs`
- existing docs and milestone index files referenced by the current test target
- the real `cargo test -p kamn-e2e-harness --test docs_contract_release_group -- --nocapture` target

Outputs:
- bounded module tree under `crates/kamn-e2e-harness/tests/docs_contract_release_group/`
- extraction contract covering the staged root budget and required module layout
- reduced root `docs_contract_release_group.rs`

# Boundaries/Non-goals

- Do not change production `kamn-e2e-harness` behavior.
- Do not weaken or delete existing release-group docs or milestone assertions.
- Do not rewrite unrelated docs harness tests outside this target.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- moved tests stop exercising the real `docs_contract_release_group` target
- docs marker or milestone issue assertions drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [ ] root test surface is extracted from `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs` into bounded sibling modules organized by release group
- [ ] root `docs_contract_release_group.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] the real `cargo test -p kamn-e2e-harness --test docs_contract_release_group -- --nocapture` target remains wired and passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs`
- `crates/kamn-e2e-harness/tests/docs_contract_release_group/**`
- `crates/kamn-e2e-harness/tests/*extraction_contract*.rs`
- `specs/6796-split-docs-contract-release-group.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing docs-contract failures remain ordinary Rust assertion failures with no silent fallbacks.
- Docs and milestone fixtures must continue to fail loudly when required files or markers are missing.

# Test plan

1. Add a red extraction contract asserting the root shell budget and required module layout.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested helpers where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-e2e-harness --test docs_contract_release_group -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6796-touched-size.json`.

# Planned module seams

- `r52_docs_contract_tests.rs` for the r52 preflight and integration-config docs checks
- `r53_docs_contract_tests.rs` for the r53 execution-activation docs checks
- `r54_r55_docs_contract_tests.rs` for the r54 evidence/teardown and r55 evidence-step/live-s02 docs checks
- `r56_r60_docs_contract_tests.rs` for the r56-r60 verification hardening docs checks
- `r61_r64_docs_contract_tests.rs` for the r61-r64 verification format docs checks
