# Plan: Issue #6207 - AgentIdentity Secret Material Zeroization + Clone Removal

## Approach

1. Add deterministic zeroization in `AgentIdentity` via explicit `Drop` implementation.
2. Remove `Clone` from `AgentIdentity` derive list.
3. Update any tests relying on `identity.clone()` to avoid clone dependency.
4. Add regression source-contract tests for clone removal + zeroization markers.

## Affected Modules

- `crates/kamn-agent-lib/src/identity.rs`
- `crates/kamn-agent-lib/src/lib.rs`

## Risks and Mitigations

- Risk: test breakage from removed clone semantics.
  - Mitigation: update tests to compare captured DID values instead of cloning identity.
- Risk: zeroization marker drift.
  - Mitigation: add include_str-based regression marker test.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p kamn-agent-lib -- -D warnings`
- `cargo test -p kamn-agent-lib -- --nocapture`
