# Spec: Issue #6233 - Migrate README Contract Lane from Shell to Rust Integration Test

- Status: Implemented
- Priority: P1
- Parent: #6223
- Milestone: R59 Swarm Gap Closure

## Problem Statement

`scripts/ci/test_readme_contract.sh` contains large inline marker arrays and implements contract validation logic in shell. This contributes avoidable shell-surface overhead and duplicates test logic that belongs in Rust integration suites.

## Scope

In scope:
- Add a Rust integration test lane that validates README contract headers and required marker snippets.
- Externalize required marker snippets into a deterministic tracked fixture file.
- Slim `scripts/ci/test_readme_contract.sh` to a thin wrapper that delegates to the Rust test lane.
- Preserve existing command entrypoint (`bash scripts/ci/test_readme_contract.sh`).

Out of scope:
- Migrating all CI helper scripts in this issue.
- Changing README marker content semantics beyond relocation into a fixture.

## Acceptance Criteria

### AC-1 Rust-Owned Contract Validation
Given README contract execution,
When the lane runs,
Then marker/header validation is performed by a Rust integration test (not inline shell arrays).

### AC-2 Compatibility-Preserving Wrapper
Given existing CI command surfaces,
When `bash scripts/ci/test_readme_contract.sh` is invoked,
Then it executes the Rust contract lane and returns equivalent pass/fail behavior.

### AC-3 Shell-Surface Reduction
Given pre/post migration shell surface,
When the issue is closed,
Then shell LOC for the migrated lane decreases and DoD shell-surface markers are reported.

## Conformance Cases

- C-01 (AC-1, Unit): Rust test enforces README headers and marker snippet inventory from tracked fixture.
- C-02 (AC-2, Integration): shell wrapper content delegates to `cargo test -p kamn-core --test readme_contract_lane` and `bash scripts/ci/test_readme_contract.sh` passes.
- C-03 (AC-3, Regression): shell LOC delta for the migrated script is negative and closure reports measured shell/rust ratio deltas.
