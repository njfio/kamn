# Plan: Issue #6203 - Reclassify `*_docs.rs` Tests into Governance Lint Surface

## Approach

1. Extend shell-test-surface policy to classify docs/governance rust tests by filename markers.
2. Exclude those files from behavioral rust test denominator and track separate count.
3. Emit docs-test count in JSON report payload.
4. Refresh baseline fixture values to match new deterministic counting model.

## Affected Modules

- `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs`
- `fixtures/ci/shell_test_surface_ratio_baseline.env`

## Risks and Mitigations

- Risk: baseline drift due changed counting semantics.
  - Mitigation: refresh baseline with deterministic local count and keep threshold policy strict.
- Risk: accidental exclusion of non-doc behavioral tests.
  - Mitigation: narrow classifier to explicit docs/governance marker patterns.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p kamn-core --test shell_test_surface_ratio_policy -- -D warnings`
- `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`
