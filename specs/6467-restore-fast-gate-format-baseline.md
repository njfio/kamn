# Spec: Issue 6467 - Restore Fast Gate format baseline

## Objective
Re-establish a passing Rust formatting baseline so `cargo fmt --all --check`
passes locally and in PR Fast Gate CI.

## Inputs/Outputs
- Inputs:
  - Rust source and test files currently failing rustfmt checks.
- Outputs:
  - Format-only diff produced by `cargo fmt --all`.
  - Local `cargo fmt --all --check` success.

## Boundaries/Non-goals
- No functional code changes.
- No dependency/workflow configuration changes.
- No test behavior changes beyond split-contract whitespace normalization needed
  to avoid rustfmt false negatives.

## Failure modes
- Formatting command misses files and CI still fails.
- Manual edits accidentally alter behavior.

## Acceptance criteria (testable booleans)
- [x] AC-1: `cargo fmt --all --check` passes locally.
- [x] AC-2: Generated changes are format-only (no behavior edits).
- [ ] AC-3: Fast Gate formatting step can pass on PR CI.

## Files to touch
- Rust files modified by `cargo fmt --all`.
- `specs/6467-restore-fast-gate-format-baseline.md`

## Error semantics
- Preserve existing runtime and test behavior.
- Treat formatting command failures as hard failures.

## Test plan
- Red:
  - Run `cargo fmt --all --check` and capture failing baseline.
  - Baseline observed failing across multiple files (for example
    `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs` and
    `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`).
- Green:
  - Run `cargo fmt --all`.
  - Re-run `cargo fmt --all --check`.
- Refactor:
  - Ensure diff remains formatter-only.
- Integration:
  - Push PR and verify Fast Gate formatting stage.

## Refactor verification
- Formatter pass produced layout/import-order updates only; no dependency,
  workflow, or runtime behavior changes were introduced in this issue.

## Phase 6 integration evidence
- Verified commands:
  - `cargo fmt --all`
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test signer_backend_split_contract`
- PR CI evidence for Fast Gate formatting stage tracked in Phase 7.

## Deviations
- Split-contract marker assertions were upgraded to whitespace-insensitive
  matching to keep delegation contract checks stable under rustfmt wrapping.
