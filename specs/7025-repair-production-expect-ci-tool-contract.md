# 7025-repair-production-expect-ci-tool-contract

## Objective
Restore the Fast Gate CI-tool regression lane by making the production panic/expect checker pass on the current branch without weakening production panic, `expect`, `unreachable!`, or unsafe environment fallback semantics.

## Inputs/Outputs
- Inputs:
  - `scripts/ci/test_ci_tools.sh`
  - `scripts/ci/test_check_no_production_expect.sh`
  - `scripts/ci/check_no_production_expect.py`
  - Rust files reported by the checker baseline.
- Outputs:
  - `bash scripts/ci/test_check_no_production_expect.sh` passes.
  - Fast-mode CI tool regression progresses past the production expect subtest.
  - Baseline checker output returns `status=ok`, `reason_codes_value=none`, and `runtime_panic_replacement_evidence_violation_count=0`.

## Boundaries/Non-goals
- Do not remove the production panic/expect checker from Fast Gate.
- Do not weaken detection for production `expect`, `panic!`, `unreachable!`, or unsafe env defaults.
- Do not classify real runtime/demo paths as test-only merely to pass the gate.
- Do not broaden this issue into MVP demo feature work.

## Failure Modes
- The checker treats test-only support modules as production and fails on legitimate test assertions.
- Real production code uses `expect`, `unreachable!`, or unsafe env fallback defaults and should continue to fail until repaired.
- The shell contract exits without enough context to identify which baseline contract failed.
- Platform-specific local shell behavior can mask the CI failure; Linux/Fast Gate evidence remains authoritative for CI.

## Acceptance Criteria
- [x] Red evidence captures `bash scripts/ci/test_check_no_production_expect.sh` failing because the baseline checker returns `status=fail`.
- [x] Test-only exclusions cover only clearly test-only surfaces such as `#[cfg(test)]` items, `src/**/tests.rs`, `src/**/runtime_tests/**`, `src/**/main_tests/**`, and named test-support directories.
- [x] Genuine production violations reported by the checker are replaced with explicit error/option/result handling rather than policy exclusions.
- [x] `bash scripts/ci/test_check_no_production_expect.sh` passes.
- [x] `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes on the CI-compatible path, or any remaining local-platform blocker is documented with exact evidence.
- [x] `cargo fmt --check`, strict workspace clippy, and `make check` remain green.

## Files To Touch
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- Rust files reported by the baseline checker that are genuine production paths.

## Error Semantics
- Production panic/expect/unsafe-default findings remain hard failures.
- Test-only classification must be deterministic and path/attribute based.
- Missing scan roots remain hard failures with `scan_root_not_found`.

## Test Plan
- Red: run `bash scripts/ci/test_check_no_production_expect.sh` and capture the baseline `status=fail` evidence.
- Green: repair checker test-only classification and production violations until the baseline path returns `status=ok`.
- Integration: rerun `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` or a CI-equivalent Linux reproduction if local macOS portability blocks the full fast-mode script.

## Completion Evidence
- Red: `bash scripts/ci/test_check_no_production_expect.sh` failed on `src/runtime_tests/support.rs` being classified as production.
- Green: `python3 scripts/ci/check_no_production_expect.py --output-json /tmp/kamn-no-production-expect-green-candidate.json` returned `status=ok` and `violation_count=0`.
- Green: `bash scripts/ci/test_check_no_production_expect.sh` passed.
- Green: `python3 -m py_compile scripts/ci/check_no_production_expect.py` passed.
- Green: `cargo fmt --check` passed.
- Green: `cargo check -p kamn-core -p kamn-e2e-harness --all-targets --all-features` passed.
- Closeout: `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed in `11m 42s`.
- Closeout: `make check` passed.
- Integration: `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` is blocked locally before this subtest by macOS `date` lacking `-d`.
- Integration: with a temporary `/opt/homebrew/bin/gdate` shim for `date`, the CI subtest sequence through `test_check_no_production_expect.sh` passed: `test_evaluate_budget`, `test_check_anti_flake_policy`, `test_anti_flake_merge_gate_policy`, and `test_check_no_production_expect`.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +200`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7025`
