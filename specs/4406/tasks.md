# Tasks — #4406

Status: Reviewed

T1
- Implement deterministic reason mapping and taxonomy marker outputs in invariant policy checker.

T2
- Add deterministic taxonomy/evidence fields to combined invariant lane summary report payload.

T3
- Update checker and lane tests for pass/fail marker expectations and tamper regressions.

T4 (Regression)
- Run:
  - `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
  - `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
  - `bash scripts/ci/test_ci_tools.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

T5
- Update release-go/no-go and invariant strategy docs with taxonomy marker contract.
