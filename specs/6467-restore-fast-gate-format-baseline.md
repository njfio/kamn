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
- No test logic changes beyond formatter output.

## Failure modes
- Formatting command misses files and CI still fails.
- Manual edits accidentally alter behavior.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `cargo fmt --all --check` passes locally.
- [ ] AC-2: Generated changes are format-only (no behavior edits).
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
- Green:
  - Run `cargo fmt --all`.
  - Re-run `cargo fmt --all --check`.
- Refactor:
  - Ensure diff remains formatter-only.
- Integration:
  - Push PR and verify Fast Gate formatting stage.

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
