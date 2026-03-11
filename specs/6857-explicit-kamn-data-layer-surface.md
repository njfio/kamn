## Objective

Replace `kamn-data-layer` glob re-exports with explicit crate-root re-exports so the public API is reviewable, intentional, and stable without leaking whole module internals by default.

## Inputs/Outputs

Inputs:
- Current crate root in `crates/kamn-data-layer/src/lib.rs`
- Public items exposed by the currently glob-exported data-layer modules
- Downstream root imports in `kamn-core` and `kamn-data-layer` tests

Outputs:
- `crates/kamn-data-layer/src/lib.rs` with explicit `pub use` lists instead of `pub use module::*;`
- A hard-fail contract test that rejects new glob re-exports in the crate root
- Downstream compile/test coverage showing the explicit surface still satisfies current uses

## Boundaries/Non-goals

Non-goals:
- Broad `kamn-data-layer` redesign
- Renaming public items without a migration reason
- Introducing new dependencies
- Changing module-local APIs that are already well-scoped under module paths

Boundaries:
- Keep changes scoped to `crates/kamn-data-layer/**` plus any narrowly required downstream import adjustments
- Preserve behavior and current compile/test semantics

## Failure Modes

- Crate root continues to expose `pub use ...::*;` and defeats API review
- Explicit export list accidentally drops items currently relied on by downstream crates
- Downstream callers compile only through broad module-path access rather than the reviewed root surface
- New explicit root surface becomes inconsistent with current data-layer tests

## Acceptance Criteria

- [x] `crates/kamn-data-layer/src/lib.rs` contains no `pub use ...::*;` re-exports
- [x] The crate root exposes explicit reviewed `pub use` lists for the intended public surface
- [x] A contract test fails if glob re-exports return
- [x] `cargo test -p kamn-data-layer -- --nocapture` passes
- [x] Relevant downstream compile/test checks pass without unrelated behavior changes

## Files To Touch

- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/tests/data_layer_public_surface_contract.rs`
- Downstream import sites only if explicit root exports require narrow adjustments

## Error Semantics

- The contract test must fail with explicit missing-marker or forbidden-glob messages
- No production runtime error semantics change in this issue
- Any downstream compile break must be handled explicitly rather than masked by restoring globs

## Test Plan

Red:
- Add a contract test asserting the crate root has no glob re-exports
- Assert the crate root includes explicit export markers for the intended public data-layer surface
- Run the contract and confirm it fails on current `main`

Green:
- Replace the crate-root globs with explicit re-export lists
- Fix any direct downstream import regressions caused by the narrowed root surface

Refactor/Integration:
- Keep `lib.rs` and the new contract under the active size limits
- Run `cargo test -p kamn-data-layer -- --nocapture`
- Run narrow downstream checks for `kamn-core` paths that import from `kamn_data_layer`

## Phase 6 Evidence

- `cargo test -p kamn-data-layer -- --nocapture`
- `cargo test -p kamn-data-layer --test data_layer_public_surface_contract -- --nocapture`
- `cargo test -p kamn-core --no-run`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6857-origin-clean-1773204886 --base-ref origin/main --output-json /tmp/6857-touched-size-clean2.json`

## Deviations

- The shell wrapper `scripts/ci/check_touched_rust_size_policy.sh` was not used for final clean-clone validation because it resolved the wrong repo root. The direct Python entrypoint was used instead against the clean clone path.
