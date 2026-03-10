# Objective

Extract the oversized test surface from `crates/kamn-core/tests/p2p_live_transport_runtime.rs` into bounded sibling modules while preserving the existing live transport runtime assertions and helper semantics.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- live transport runtime helpers embedded in the root file
- real runtime/p2p harness wiring exercised by the target

Outputs:
- bounded module tree under `crates/kamn-core/tests/p2p_live_transport_runtime/`
- extraction contract covering the new layout and staged root budget
- reduced root `p2p_live_transport_runtime.rs`

# Boundaries/Non-goals

- Do not change live transport runtime production behavior or marker semantics.
- Do not broaden the issue outside `p2p_live_transport_runtime.rs` and its new sibling modules.
- Do not rewrite unrelated `kamn-core` harnesses.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- helper moves break the live transport runtime harness or test determinism
- existing assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-core/tests/p2p_live_transport_runtime.rs` into bounded sibling modules
- [x] root `p2p_live_transport_runtime.rs` is reduced below the staged extraction cap enforced by the new contract
- [x] extracted sibling files stay within the active file-size budget
- [x] `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture` passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6786-split-p2p-live-transport-runtime.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Existing live transport runtime failures remain ordinary Rust assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6786-touched-size.json`.

# Evidence

- Root shell reduced to `12` LOC at `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- Verification commands:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test p2p_live_transport_runtime_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6786-touched-size-final.json`
- Final touched-Rust result: `policy_decision=GO`

# Deviations

- None
