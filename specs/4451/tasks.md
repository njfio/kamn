# Tasks: Issue #4451

Status: Completed
Issue: #4451

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh` with
  failing assertions for normalized reason-value and parity evidence-output markers.
- Run:
  - `bash scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
- Expect RED before implementation updates.

T2 (RED, Integration/Conformance):
- Extend `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
  with failing assertions for normalized reason/evidence markers.
- Run:
  - `bash scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
- Expect RED before implementation updates.

T3 (GREEN, Implementation):
- Implement reason mapper and parity evidence-output normalization in
  `scripts/runtime/local_full_stack_integration_live_contract.py`.
- Wire normalized markers through
  `scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh`.

T4 (GREEN, Docs/Regression):
- Update `docs/architecture/runtime.md` and
  `crates/kamn-core/tests/runtime_architecture_docs.rs` with new deterministic references.
- Run:
  - `cargo test -p kamn-core --test runtime_architecture_docs`

T5 (Verify, Regression):
- Re-run targeted tests and scoped hygiene:
  - `bash scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
  - `bash scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
  - `cargo test -p kamn-core --test runtime_architecture_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
    - Failed with: `expected local full-stack integration policy normalized reason_codes_value marker`
  - `bash scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
    - Failed with: `expected local full-stack integration contract lane policy normalized reason_codes_value marker`
  - `cargo test -p kamn-core --test runtime_architecture_docs`
    - Failed with:
      - `assertion failed: DOC.contains("runtime_phase_parity_reason_codes_value=<normalized runtime extraction reason key>")`
- GREEN command/output:
  - `bash scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
    - Passed: `local full-stack integration policy checker tests passed.`
  - `bash scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
    - Passed: `local full-stack integration contract lane tests passed.`
  - `cargo test -p kamn-core --test runtime_architecture_docs`
    - Passed: `7 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed
- Regression summary:
  - Runtime extraction policy now emits deterministic normalized reason mapping and
    deterministic parity evidence-output markers while preserving detailed fail-closed
    diagnostics through `failed_checks`.
