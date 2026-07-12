# Issue 7113: Refresh Test File Inventory Baseline

## Objective

Restore the test-file size policy contract by refreshing its tracked inventory
total to the current repository state while preserving every size threshold,
budget, offender rule, and fail-closed assertion.

## Inputs And Outputs

Input:

- Tracked `crates/**/*.rs` files containing a `/tests/` path component.
- The policy exclusion for `/tests/support/` paths.
- Existing thresholds and baseline fixture.

Output:

- A baseline fixture whose inventory and oversized counts equal the recomputed
  repository state.
- An unchanged policy test and threshold configuration.

## Boundaries And Non-Goals

- Do not change test discovery, exclusions, thresholds, maximum budgets, reason
  codes, severe allowlist, or Rust policy assertions.
- Do not delete, split, or weaken tests to reduce the count.
- Refresh only values proven stale by current inventory evidence.
- This issue does not repair unrelated workspace gates.

## Failure Modes

- Baseline total differs from recomputed tracked test inventory.
- Soft, severe, or hard counts are refreshed without recomputation.
- A severe/hard offender or budget breach is hidden by fixture changes.
- Schema, first-wave split markers, or allowlist changes accidentally.

## Error Semantics

The existing Rust policy remains authoritative and fail-loud. The fixture is
declarative evidence only; no fallback or tolerance is introduced.

## Acceptance Criteria

- [ ] Baseline `test_file_total` equals the recomputed current inventory.
- [ ] Soft, severe, and hard counts match current files.
- [ ] Severe allowlist remains exact and no hard offender exists.
- [ ] Threshold and Rust policy files remain unchanged.
- [ ] `test_file_size_policy`, formatting, strict clippy, `make check`, and
      `make test` pass.

## Files To Touch

- `fixtures/ci/test_file_size_policy_baseline.env`
- This spec only.

## Test Plan

RED is the existing committed policy failure:

```text
test file inventory drift
left: 1317
right: 1288
```

GREEN verification:

```bash
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-core \
  --test test_file_size_policy -- --nocapture
cargo fmt --check
CARGO_TARGET_DIR=target/mvp-demo-proof cargo clippy \
  --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=target/mvp-demo-proof make check
CARGO_TARGET_DIR=target/mvp-demo-proof make test
```
