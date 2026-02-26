# Plan: Issue #6009

## Approach
1. Reproduce failing gate locally to capture RED state and current measured counts.
2. Update `fixtures/ci/shell_test_surface_ratio_baseline.env` with refreshed `shell_test_file_count`, `rust_test_file_count`, `shell_to_rust_ratio`, and `refreshed_on`.
3. Re-run shell test surface ratio gate and confirm GREEN.
4. Verify thresholds remain strict and waiver remains disabled.

## Affected Modules
- `fixtures/ci/shell_test_surface_ratio_baseline.env`
- Validation-only: `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs` (no code changes expected)

## Risks / Mitigations
- Risk: Baseline value drift if counting method is inconsistent.
  Mitigation: Use counts emitted by failing gate output and immediate local re-run confirmation.
- Risk: Accidental policy weakening.
  Mitigation: No edits to `.ci/shell_test_surface_ratio_thresholds.env`; confirm unchanged via tests.

## Interfaces / Contracts
- No API or runtime behavior change.
- Only CI contract fixture values are refreshed.
