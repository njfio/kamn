# Objective

Extract the oversized test surface from `crates/kamn-core/tests/kolme_runtime_commit_client.rs` into bounded sibling modules while preserving the existing runtime commit client contract assertions and helper semantics.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/kolme_runtime_commit_client.rs`
- fixture file `fixtures/kolme_commit/runtime_commit_request_cases.txt`
- runtime commit client test doubles and helper parsing now embedded in the root file

Outputs:
- bounded module tree under `crates/kamn-core/tests/kolme_runtime_commit_client/`
- extraction contract covering the new layout and staged root budget
- reduced root `kolme_runtime_commit_client.rs`

# Boundaries/Non-goals

- Do not change runtime commit client production behavior or marker semantics.
- Do not broaden the issue outside `kolme_runtime_commit_client.rs` and its new sibling modules.
- Do not rewrite unrelated `kamn-core` harnesses.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- helper moves break fixture loading or typed error expectations
- existing runtime commit client assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-core/tests/kolme_runtime_commit_client.rs` into bounded sibling modules
- [x] root `kolme_runtime_commit_client.rs` is reduced below the staged extraction cap enforced by the new contract
- [x] extracted sibling files stay within the active file-size budget
- [x] `cargo test -p kamn-core --test kolme_runtime_commit_client -- --nocapture` passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/kolme_runtime_commit_client.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_client/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6784-split-kolme-runtime-commit-client.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Existing runtime commit client harness failures remain ordinary Rust assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test kolme_runtime_commit_client -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6784-touched-size.json`.

# Phase 6 Evidence

- Root shell reduced to `5` module declarations in `crates/kamn-core/tests/kolme_runtime_commit_client.rs`.
- Extracted module tree under `crates/kamn-core/tests/kolme_runtime_commit_client/` covers:
  - request validation
  - adapter-backed client behavior
  - live provider behavior
  - finality checker behavior
  - shared support fixtures and transports
- Verified with:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test kolme_runtime_commit_client_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test kolme_runtime_commit_client -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6784-remote-size-post-refactor.json`
- Final touched-Rust result: `policy_decision=GO`

# Deviations

- The first “fresh” clone was invalid because it was cloned from the dirty local checkout rather than from the remote repository; the final verification and merge branch used `/tmp/kamn-6784-remote` cloned directly from `https://github.com/njfio/kamn.git`.
