# Issue 7109: Stabilize Governance Ratio Head Contract

## Objective

Keep the current-head governance ratio contract aligned with the actual policy
instead of one historical 50-commit composition. A compliant branch head must
remain green when its exact governance and feature counts change but all report
invariants and the configured threshold still hold.

## Inputs And Outputs

Input:

- The governance feature-commit ratio report produced for `HEAD` from the fixed
  activation base, 50-commit window, and `0.20` maximum governance ratio.

Output:

- A passing Rust contract when the checker succeeds and its report is internally
  consistent, complete, and policy-compliant.
- A failing contract when checker status, totals, ratios, or threshold compliance
  are invalid.

## Boundaries And Non-Goals

- Do not change the Python checker, commit classification, activation base,
  threshold, window size, CI configuration, or shell surface.
- Do not weaken synthetic threshold success/failure or exact classification tests.
- Do not encode a new expected historical commit composition.
- This issue repairs only the stale Rust current-head assertion.

## Failure Modes

- Checker process failure remains a hard test failure with stdout evidence.
- A non-`ok` report status fails.
- A report whose governance, feature, and unknown counts do not equal the
  non-merge total fails.
- A report whose governance or feature ratio differs from its reported counts
  fails.
- A governance ratio above the configured maximum fails.
- A window total other than 50 fails.

## Error Semantics

The test uses exact assertions for status and window size, relational assertions
for counts, and floating-point tolerance only for ratios derived from integer
counts. Checker failures are never converted to success or ignored.

## Acceptance Criteria

- [ ] Current `main` passes without hard-coded governance or feature counts.
- [ ] The test requires checker success and `status=ok`.
- [ ] Governance, feature, and unknown counts sum to the non-merge total.
- [ ] Reported governance and feature ratios equal ratios derived from counts.
- [ ] Governance ratio is at or below `0.20` and the total remains 50.
- [ ] Synthetic threshold and classification contracts remain green unchanged.
- [ ] Formatting, strict workspace clippy, the full governance contract binary,
      and `make check` pass.

## Files To Touch

- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/current_head_status_contract_tests.rs`
- This spec only.

## Test Plan

RED is the existing merged-main failure:

```text
assertion left == right failed
left: 3
right: 10
```

GREEN verification:

```bash
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-core \
  --test governance_feature_commit_ratio_base_compliance \
  current_head_status_contract_tests::current_branch_head_restores_ratio_compliance \
  -- --exact --nocapture
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-core \
  --test governance_feature_commit_ratio_base_compliance
cargo fmt --check
CARGO_TARGET_DIR=target/mvp-demo-proof cargo clippy \
  --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=target/mvp-demo-proof make check
```
