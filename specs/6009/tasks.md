# Tasks: Issue #6009

## Ordered Tasks
- T1 (RED): Reproduce failing gate `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`.
- T2 (Implementation): Refresh baseline fixture counts and ratio.
- T3 (GREEN): Re-run shell surface ratio gate test and confirm pass.
- T4 (Regression): Verify thresholds and waiver settings are unchanged and strict.

## Tier Mapping
- Unit: T1, T3
- Functional: T1, T3
- Regression: T4
- Conformance: T3
