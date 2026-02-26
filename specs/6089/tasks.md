# Tasks: Issue #6089

## Ordered Tasks
- T1 (RED/Conformance): Capture baseline duplicate-family inventory and shell LOC; add parity assertion that fails if wrappers still embed dispatcher body.
- T2 (Implementation): Rewrite bounded duplicate wrapper family to thin delegating wrappers with unchanged filenames.
- T3 (Verification): Run wrapper parity + stale-reference checks for migrated family.
- T4 (Verification): Compute and record `shell_loc_delta_actual`, `rust_loc_delta_actual`, `shell_to_rust_ratio_delta_actual`, and `shell_surface_ratio_target_status`.
- T5 (Closure): Publish shell-surface DoD markers with mitigation linkage if ratio regresses.

## Tier Mapping
- Functional: T3
- Regression: T3
- Conformance: T1, T4, T5
- Unit: N/A (shell-surface change)
- Integration: N/A (no runtime integration surface change)
