# Tasks — #4401

Status: Reviewed

T1 (RED)
- Add failing policy checker tests for:
  - lane-failure acceptance mismatch;
  - missing/tampered invariant taxonomy/evidence markers.

T2 (GREEN)
- Implement deterministic reason mapping and marker emission in `check_invariant_fuzz_concurrency_policy.sh`.
- Add normalized taxonomy/evidence fields to invariant/fuzz/concurrency lane summary payload.

T3 (Regression)
- Run:
  - `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
  - `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
  - `bash scripts/ci/test_ci_tools.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

T4
- Update docs and finalize AC mapping for PR.
