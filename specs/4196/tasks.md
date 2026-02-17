# Tasks — #4196 Deterministic Full-Stack Harness Checker Reason Outputs

## Ordered Tasks
- T1 (Regression/Functional): extend `scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh` with deterministic output assertions and repeated-run stability checks.
- T2 (Implementation): update `scripts/runtime/full_io_scenario_matrix_live_contract.py` policy checker to emit deterministic taxonomy/codes markers and fail-closed reason-code values.
- T3 (Docs): add release checklist gate section to `docs/foundation/release-gonogo-checklist.md`.
- T4 (Docs Contract): add assertions in `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`.
- T5 (Verify): run targeted checker and docs-contract test suites.

## Test Tier Mapping
- Unit: N/A (script/documentation contract scope)
- Functional: checker pass-path deterministic output markers
- Integration: checker invocation via shell contract tests
- Regression: tampered harness marker/parity mismatch fail-closed reason mapping
- Performance: N/A (bounded CI smoke-only path)
