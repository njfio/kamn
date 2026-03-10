# Objective

Extract the oversized docs-contract surface from `crates/kamn-core/tests/kolme_devnet_ops_docs.rs` into bounded sibling modules while preserving the existing assertions against `docs/planning/kolme-devnet-ops.md` and `docs/deploy/kolme_devnet_ops.md`.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `docs/planning/kolme-devnet-ops.md`
- `docs/deploy/kolme_devnet_ops.md`

Outputs:
- bounded module tree under `crates/kamn-core/tests/kolme_devnet_ops_docs/`
- extraction contract covering the new layout and staged root budget
- reduced root `kolme_devnet_ops_docs.rs`

# Boundaries/Non-goals

- Do not change devnet ops documentation semantics.
- Do not broaden the issue outside `kolme_devnet_ops_docs.rs` and its new sibling modules.
- Do not rewrite unrelated docs-contract test files.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- the docs-contract target no longer compiles because module markers or helper visibility drift
- existing devnet ops assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] root docs-contract tests are extracted from `crates/kamn-core/tests/kolme_devnet_ops_docs.rs` into bounded sibling modules
- [ ] root `kolme_devnet_ops_docs.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test kolme_devnet_ops_docs -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6776-split-kolme-devnet-ops-docs.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Runtime test failures remain ordinary Rust test assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and a staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test kolme_devnet_ops_docs -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6776-touched-size.json`.

# Planned module seams

- `service_api_failover_contract_tests.rs`
- `deploy_compat_contract_tests.rs`
- `local_lane_contract_tests.rs`
- `migration_manifest_contract_tests.rs`
- `regression_migration_contract_tests.rs`
- `regression_local_lane_contract_tests.rs`
- `runtime_transport_contract_tests.rs`

# Phase 6 Evidence

- `cargo test -p kamn-core --test kolme_devnet_ops_docs_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test kolme_devnet_ops_docs -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6776-touched-size-refactor.json`
- Root shell result: `crates/kamn-core/tests/kolme_devnet_ops_docs.rs` = `19` LOC.
- Largest extracted file in the staged write set: `deploy_compat_contract_tests/compatibility_marker_contract_tests.rs` = `175` LOC.
- Touched-Rust result: `policy_decision=GO`.

# Deviations

- No behavioral deviations from the original docs-contract surface.
- The local-lane and regression-local seams required second-level module splits to keep every touched file within the active 200 LOC cap.
