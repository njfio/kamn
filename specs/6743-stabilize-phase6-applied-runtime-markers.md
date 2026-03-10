# 6743 - Stabilize phase6 applied runtime markers

## Objective
Make `functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output` deterministic under full `daemon_tests` execution by selecting the correct daemon completion log for the applied path instead of relying on ambiguous suite ordering.

## Inputs/Outputs
- Input: daemon runtime contract tests and log-selection helpers used by phase6 projection assertions
- Output: deterministic applied-path selection plus regression coverage that stays green in isolation and under the full suite

## Boundaries/Non-goals
- Do not fold topology-mapping extraction work into this issue
- Do not redesign daemon runtime semantics
- Do not weaken existing phase6 projection assertions

## Failure modes
- The applied-path test still passes in isolation but fails under full `daemon_tests`
- Selection logic binds to a foreign completion log with the wrong execution id or phase6 reason code
- Regression coverage does not prove the suite-wide ordering is now deterministic

## Acceptance criteria
- [ ] `cargo test -p kamn-node daemon_tests -- --nocapture` passes on the blocker branch
- [ ] `functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output` passes both in isolation and under the full suite
- [ ] Applied-path assertions bind to the intended daemon completion log deterministically
- [ ] Regression coverage proves the selector ignores foreign completion logs that would otherwise satisfy looser matching

## Files to touch
- `specs/6743-stabilize-phase6-applied-runtime-markers.md`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/phase6_projection_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/support.rs`
- any narrowly scoped daemon runtime test file needed for the regression

## Error semantics
- Test helpers must fail closed with explicit assertion messages when the expected execution-specific completion log is absent
- No fallback to first-match completion logs

## Test plan
- Add a red regression covering foreign completion logs versus the target applied-path execution
- Fix the applied-path selector to bind deterministically
- Run the isolated applied-path test
- Run `cargo test -p kamn-node daemon_tests -- --nocapture`

## Phase 6 evidence
- Isolated regression: `cargo test -p kamn-node regression_runtime_daemon_applied_phase6_log_selection_uses_execution_id -- --nocapture`
- Isolated applied-path contract: `cargo test -p kamn-node functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --nocapture`
- Real suite path: `cargo test -p kamn-node daemon_tests -- --nocapture`
- Touched-Rust ratchet: `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6742-baseline-VGAsi1 --base-ref origin/main --output-json /tmp/6743-touched-size.json`
- Ratchet result: `policy_decision=GO`

## Deviations
- The applied-path test now captures JSON and text renders through the same dedicated chain id (`phase6-applied-contract`) so completion-log assertions bind to the intended execution consistently under full-suite ordering.
