# Objective

Extract the oversized test surface from `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs` into bounded sibling modules while preserving the existing shell/workflow/docs contract assertions and command-execution behavior.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs`
- referenced workflows, scripts, docs, and fixtures exercised by the test target

Outputs:
- bounded module tree under `crates/kamn-core/tests/shell_test_surface_migration_wave1/`
- shared support module for temporary directories, process execution, and assertion helpers
- extraction contract covering the new layout and staged root budget
- reduced root `shell_test_surface_migration_wave1.rs`

# Boundaries/Non-goals

- Do not change shell-surface migration semantics or workflow/docs policy markers.
- Do not broaden the issue outside `shell_test_surface_migration_wave1.rs` and its new sibling modules.
- Do not rewrite unrelated test targets.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- helper visibility or module markers drift and the target no longer compiles
- existing shell/workflow/docs assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] root test surface is extracted from `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs` into bounded sibling modules
- [ ] root `shell_test_surface_migration_wave1.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test shell_test_surface_migration_wave1 -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs`
- `crates/kamn-core/tests/shell_test_surface_migration_wave1/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6778-split-shell-test-surface-migration-wave1.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Existing command-execution test failures remain ordinary Rust assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test shell_test_surface_migration_wave1 -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6778-touched-size.json`.

# Planned module seams

- `support.rs`
- `ci_exclusion_contract_tests.rs`
- `workflow_policy_contract_tests.rs`
- `wrapper_parity_contract_tests.rs`
- `command_contract_tests.rs`
- `service_api_contract_tests.rs`

# Phase 6 evidence

- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test shell_test_surface_migration_wave1_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test shell_test_surface_migration_wave1 -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6778-remote-size-go3.json`

# Deviations

- Verification moved to isolated clone `/tmp/kamn-6778-remote` because linked worktrees inherited unrelated tracked edits from the primary repo and made the touched-Rust gate non-authoritative for `#6778`.
- Formatting used targeted `rustfmt` on the `#6778` write set instead of `cargo fmt --all`; full-workspace formatting in the dirty local environment broadened the touched set beyond the issue scope.
