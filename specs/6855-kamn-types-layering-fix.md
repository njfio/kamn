# Objective
Remove the `kamn-types -> kamn-core` dependency inversion by moving the first-wave DID value surface into `kamn-types`, making `kamn-core` consume `kamn-types`, and preserving compatibility for downstream callers through temporary `kamn-core` re-export shims.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-types/Cargo.toml`
  - `crates/kamn-types/src/lib.rs`
  - `crates/kamn-core/src/did.rs`
  - `crates/kamn-core/src/lib.rs`
  - downstream callers in `kamn-core`, `kamn-sdk`, `kamn-node`, and `kamn-governance`
  - architecture guidance in `docs/architecture/kamn-core-target-crate-graph.md` and `docs/architecture/kamn-types.md`
- Outputs:
  - `kamn-types` owns the first-wave DID value surface directly
  - `kamn-types` no longer depends on `kamn-core`
  - `kamn-core` depends on `kamn-types` for the moved DID/value surface
  - temporary compatibility re-exports keep existing downstream users compiling
  - tests/contracts document and enforce the inverted dependency direction

## Boundaries/Non-goals
- Do not redesign broader `kamn-core` decomposition beyond the DID/value first wave.
- Do not remove temporary `kamn-core` compatibility re-exports in this issue.
- Do not change DID wire format, parse semantics, or error reason text except where needed to preserve existing behavior under the new ownership boundary.
- Do not add new third-party dependencies.
- Do not move runtime registry/orchestration logic out of `kamn-core` in this issue.

## Failure modes
- `kamn-types` still depends on `kamn-core` after the move.
- `kamn-core` and `kamn-types` form a circular dependency.
- Downstream crates stop compiling because the compatibility shim surface is incomplete.
- DID parsing or key-binding behavior drifts.
- Public types move but documentation/contracts still claim the old temporary re-export state.
- Extracted files or functions violate the active size policy during the inversion.

## Acceptance criteria
- [ ] `crates/kamn-types/Cargo.toml` no longer declares `kamn-core` as a dependency.
- [ ] The first-wave DID/value surface (`AgentDid`, `KamnDid`, `DidDocument`, `DidService`, `DidVerificationMethod`, `AgentDidError`, `KamnDidError`, `AgentDidKeyBindingError`, `AgentDidMetadata`, canonical parse helpers) is owned by `kamn-types`.
- [ ] `kamn-core` consumes the moved surface from `kamn-types` and preserves temporary compatibility re-exports for existing downstream users.
- [ ] `cargo tree -p kamn-types` no longer shows `kamn-core` in the dependency tree.
- [ ] Existing downstream users in `kamn-core`, `kamn-sdk`, `kamn-node`, and `kamn-governance` compile and their relevant test targets stay green.
- [ ] Architecture/docs contracts are updated from `temporary-kamn-core-reexport` current-state markers to the new post-inversion state where appropriate.
- [ ] No new circular dependency is introduced.

## Files to touch
- `specs/6855-kamn-types-layering-fix.md`
- `crates/kamn-types/Cargo.toml`
- `crates/kamn-types/src/lib.rs`
- `crates/kamn-core/src/did.rs`
- `crates/kamn-core/src/lib.rs`
- contract/docs tests covering `kamn-types` architecture and dependency policy
- optional downstream import sites in `crates/kamn-core`, `crates/kamn-sdk`, `crates/kamn-node`, `crates/kamn-governance`

## Error semantics
- DID parse helpers remain fail-closed.
- Existing typed DID errors remain structured and machine-distinguishable.
- Compatibility shims must not silently rewrite or weaken invalid input behavior.
- If a moved type or helper cannot be re-exported compatibly, the build must fail loudly rather than partially hiding the mismatch.

## Test plan
1. Add a red contract that fails while `kamn-types` still depends on `kamn-core` and while the first-wave DID surface is not owned by `kamn-types`.
2. Add/adjust docs contracts so the old `temporary-kamn-core-reexport` current-state marker fails until the architecture docs are updated.
3. Move the first-wave DID/value surface into `kamn-types` and invert `kamn-core` to consume it.
4. Re-run `cargo tree -p kamn-types` and the new extraction/dependency contracts until green.
5. Re-run targeted downstream test targets from `kamn-types`, `kamn-core`, `kamn-sdk`, `kamn-node`, and `kamn-governance` that exercise DID parsing/import boundaries.
6. Run the touched-Rust size ratchet on the final write set.
