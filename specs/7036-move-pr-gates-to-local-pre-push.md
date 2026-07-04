# Move PR Gates To Local Pre-Push

## Objective

Move the heavyweight `ci-fast-gate` PR enforcement out of GitHub Actions while
preserving the same gate intent as an explicit local pre-push command. The
change exists because the GitHub Fast Gate job is cancelled by the hosted
runtime budget even after the local quality gate and longer Workspace
Pre-Merge lane prove the code path.

## Inputs/Outputs

Inputs:

- Issue #7036.
- PR #7022 gate-recovery branch.
- `.github/workflows/ci-fast-gate.yml`, currently defining the Fast Gate,
  CI Tool Regression Gate, and Workspace Pre-Merge Gate PR jobs.
- Existing local verification commands:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `make check`
  - `make ci-tools`
  - `cargo test --workspace --locked --all-features --no-fail-fast`
  - critical-path coverage and mutation gate scripts.

Outputs:

- No GitHub Actions `pull_request` trigger that schedules `ci-fast-gate` PR
  jobs for new PR heads.
- A local make target that runs the pre-push gate sequence explicitly.
- Contract coverage that fails before the workflow removal/local target change
  and passes after it.
- Updated issue/spec evidence for local verification.

## Boundaries/Non-goals

- Do not add MVP demo features in this issue.
- Do not change MVP proof-report schemas or claim taxonomy.
- Do not remove the local gate scripts that the GitHub workflow previously
  called.
- Do not weaken Rust tests, lint levels, clippy strictness, formatting checks,
  critical-path proof scripts, or source-marker assertions.
- Do not fake or simulate settlement, escrow, exchange, or asset movement
  claims.
- Do not push directly to `main`.

## Failure Modes

- The GitHub `ci-fast-gate` workflow remains present and continues scheduling
  PR gate jobs on new heads.
- The local pre-push command omits strict formatting, clippy, CI-tool
  regression, workspace test, touched-size, coverage, or mutation checks.
- The local command returns success after a failed gate.
- Existing proof and policy scripts are deleted or relaxed instead of being
  moved behind the local command.
- Documentation or contract tests still describe the heavyweight gates as
  GitHub-enforced.
- Aggregate local proof lanes reuse the shared `target/debug` tree for
  repeated Cargo invocations and can hang or contaminate local pre-push proof
  evidence.
- Local-heavy readiness probes call network endpoints without per-call
  timeouts and can wedge the aggregate local proof run before lane-level
  runtime budgets fire.

## Acceptance Criteria

- [x] `ci-fast-gate` no longer has a GitHub Actions `pull_request` trigger.
- [x] A documented local command, `make pre-push`, runs the gate sequence before
      publishing changes.
- [x] `make pre-push` includes formatting, strict clippy, `make ci-tools`,
      full workspace tests, touched Rust size policy, critical-path coverage,
      and critical-path mutation checks.
- [x] Tests/contracts fail before the workflow/local target change and pass
      after it.
- [x] `cargo fmt --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
      passes.
- [x] `make check` passes.
- [x] The local pre-push command passes or any failure is separately filed with
      evidence.
- [x] Rust-heavy local proof lanes that run inside `make ci-tools` use isolated
      Cargo target directories or focused prebuild paths rather than wedging
      on the shared workspace target directory.
- [x] Local-heavy readiness probes use bounded per-attempt network calls so
      endpoint startup failures produce explicit NO-GO evidence instead of
      hanging the local gate.

## Files To Touch

- `specs/7036-move-pr-gates-to-local-pre-push.md`
- `Makefile`
- `.github/workflows/ci-fast-gate.yml`
- Focused workflow/local-gate contract tests under `scripts/ci/**` or existing
  Rust workflow contract tests.
- Docs or tests that explicitly require `ci-fast-gate` to remain GitHub
  scheduled.

## Error Semantics

- The local pre-push command must fail on the first failed gate command.
- The local pre-push command must not swallow child-process failures.
- Existing shell gate scripts must keep their current fail-closed behavior and
  success markers.
- Removing GitHub scheduling must not turn proof failures into warnings.

## Test Plan

Red:

- Add or update a contract that expects `.github/workflows/ci-fast-gate.yml` to
  be absent or inert and observes the current workflow file as a failure.
- Add or update a contract that expects `make pre-push` to include every local
  gate command and observes the current missing target as a failure.
- Add or update focused contracts for any local-heavy lane found to reuse the
  shared workspace Cargo target during aggregate pre-push proof runs.

Green:

- Remove GitHub `ci-fast-gate` workflow scheduling by deleting the workflow
  file or making it inert outside GitHub PR checks.
- Add `make pre-push` with the required local gate sequence.
- Update only tests/docs that encode the old GitHub-enforced policy.

Refactor:

- Keep the local target as a thin orchestration layer over existing scripts and
  make targets.
- Do not duplicate workflow script logic in the Makefile when an existing
  script already owns the check.
- Keep target isolation local to the lane wrapper/implementation; do not change
  proof assertions or skip Cargo tests to make the aggregate gate complete.

Integration/Proof:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `make check`
- `make ci-tools`
- Targeted contract tests added for this issue.
- `make pre-push`, or documented NO-GO evidence if an existing pre-push check
  is already failing for a separately filed reason.

## Red Evidence

- `bash scripts/ci/test_local_pre_push_gate_policy.sh` fails before
  implementation with:
  `ci-fast-gate GitHub workflow must be removed; run local gates with make pre-push`

## Green Evidence

- `bash scripts/ci/test_local_pre_push_gate_policy.sh` passes.
- `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract
  spec_c02_ci_fast_gate_workspace_premerge_job_is_not_pr_scheduled -- --exact
  --nocapture` passes: `1 passed`.
- `bash scripts/ci/test_workflow_scope_policy.sh` passes.
- `bash scripts/ci/test_workflow_runtime_ceiling_policy.sh` passes.
- `bash scripts/ci/test_workflow_retry_policy.sh` passes.
- `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` passes.
- `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract
  -- --nocapture` passes: `10 passed`.
- `make -n pre-push` shows the local pre-push gate sequence.
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passes; runtime: `4m 47s`.
- `make check` passes; runtime: `9m 11s`.

## Refactor Evidence

- `make pre-push` now reuses `$(MAKE) check` instead of duplicating the
  fmt/clippy commands.
- `bash scripts/ci/test_local_pre_push_gate_policy.sh` passes after retargeting
  the contract to verify both `pre-push` and `check`.
- `make -n pre-push` expands through `make check`, `make ci-tools`, full
  workspace tests, touched-size policy, critical-path coverage, and
  critical-path mutation.
- `bash -n scripts/ci/test_local_pre_push_gate_policy.sh` passes.
- `cargo fmt --check` passes.

## Integration Evidence

- Initial `make pre-push` failed inside `make ci-tools` at
  `scripts/runtime/test_check_service_api_axum_ingress_live_evidence_convergence.sh`
  because the default Homebrew `python3.14` did not provide `cryptography`.
- The local machine already has `/usr/bin/python3` resolving to Xcode Python
  with `cryptography 48.0.1`; `make pre-push` now selects an existing
  cryptography-capable Python and prepends it to PATH for the local gate run.
- `bash scripts/ci/test_local_pre_push_gate_policy.sh` passes after adding the
  Python preflight marker.
- `make -n pre-push` shows `/usr/bin/python3` for the cryptography preflight
  and PATH-prefixed local gate commands.
- `git diff --check` passes after the Python selection change.
- A second `make pre-push` run passed the earlier `cryptography` boundary and
  failed later in `scripts/ci/check_workspace_license_policy.py` because
  `/usr/bin/python3` lacks `tomllib`; the selector now requires both
  `cryptography` and `tomllib` and prefers Homebrew Python 3.12 before the
  Xcode fallback.
- `make -n pre-push` now selects
  `/opt/homebrew/opt/python@3.12/bin/python3.12` and PATH-prefixes the local
  gates with `/opt/homebrew/opt/python@3.12/bin/`.
- A later `make pre-push` run reached
  `scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh`
  and failed silently because the wrapper captured the child contract lane under
  `set -e`; focused replay showed the nested Python SDK unittest used the
  dotted `tests.python...` import path, which collides with an installed
  `tests` package under Homebrew Python 3.12.
- The reason-code validator now invokes the SDK assertion through
  `python3 -m unittest discover -s tests/python -p test_sdk.py -k
  test_regression_backend_adapter_errors_and_invalid_payloads_fail_closed`,
  and the wrapper prints captured child stdout/stderr before returning a
  nonzero lane status.
- `PATH=/opt/homebrew/opt/python@3.12/libexec/bin:$PATH bash
  scripts/runtime/validate_service_api_reason_code_compatibility_live.sh
  --output-json /tmp/service-api-reason-code-debug.json --max-seconds 240`
  passes.
- `PATH=/opt/homebrew/opt/python@3.12/libexec/bin:$PATH bash
  scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh`
  passes.
- The adjacent `ci-tools` service API segment from serde parity through
  validation negative-matrix contract lanes passes under the same PATH.
- A subsequent `make pre-push` run reached the Kolme signature parity matrix
  and showed `cargo test -p kamn-node
  integration_kolme_live_signer_vector_probe_contract -- --nocapture` walking
  unrelated `kamn-node` integration test binaries instead of the one vector
  probe, making the local-only gate too slow to use as a pre-push proof.
- `scripts/kolme/run_signature_parity_matrix.py` now targets the exact binary
  unit-test harness with `cargo test -p kamn-node --bin kamn-node
  main_tests::signer_tests::signer_direct_profile_contract_tests::direct_signature_contract_tests::integration_kolme_live_signer_vector_probe_contract
  -- --exact --nocapture`.
- `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` first failed
  with a missing focused-command marker, then passed after the runner was
  narrowed to the binary test harness.
- `python3 scripts/kolme/run_signature_parity_matrix.py --fixture
  fixtures/kolme_commit/signature_parity_vectors.json --max-cases 1
  --output-json /tmp/signature-parity-debug.json` passes.
- `bash scripts/kolme/test_run_signature_parity_matrix.sh` passes.
- `bash scripts/kolme/test_run_signature_parity_contract_lane.sh` passes.
- A later `make pre-push` run passed strict clippy in `14m 50s`, then failed in
  `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
  because `validate_service_api_axum_ingress_live.sh` counted
  `cargo build --quiet -p kamn-node` against the 180-second live validation
  runtime budget and exited at `233s`.
- The axum ingress validation script still builds by default for standalone
  validation, but now supports
  `KAMN_SERVICE_API_AXUM_INGRESS_SKIP_BUILD=1` only when the expected prebuilt
  `target/debug/kamn-node` binary exists.
- The axum ingress contract lane now prebuilds `kamn-node` before entering the
  shared timed lane runner and invokes validation with the explicit prebuilt
  binary path, keeping the live probe budget focused on runtime validation.
- `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
  first failed with `expected service api axum ingress validation script to
  support a prebuilt-node path`, then passed after the prebuilt-node wiring.
- A later `make pre-push` run passed the axum ingress lane, then reached
  `scripts/kolme/test_validate_continuous_runtime_commit_live.sh` and exposed
  another broad `cargo test -p kamn-node -- <filters>` command that walked the
  package test surface instead of running the three intended binary unit tests.
- `scripts/kolme/contracts/continuous_runtime_commit_contract_lane.py` now runs
  each intended test with `cargo test -p kamn-node --bin kamn-node <full-test>
  -- --exact --nocapture` and sums the pass-count markers across the focused
  runs.
- `bash scripts/kolme/test_run_continuous_runtime_commit_contract_lane.sh`
  first failed with `expected continuous runtime commit implementation marker:
  --bin`, then passed after the runner was narrowed.
- `bash scripts/kolme/test_validate_continuous_runtime_commit_live.sh` passes.
- A later aggregate `make ci-tools` run failed at
  `scripts/kolme/test_run_local_signed_to_kolme_demo_contract_lane.sh` after
  the localhost signed demo exceeded its default budget inside the local-heavy
  aggregate path.
- `bash scripts/sdk/test_run_localhost_signed_demo_contract_lane.sh` first
  failed with `expected localhost signed demo contract lane to prebuild
  examples before timed demo run`, then passed after the lane prebuilt the
  listener/sender examples and ran the timed demo with
  `KAMN_LOCALHOST_SIGNED_DEMO_SKIP_BUILD=true`.
- A later aggregate `make ci-tools` run failed at
  `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
  because replay-sensitive fixed nonces returned `409` instead of the expected
  request-validation status.
- `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
  first failed with `expected service api axum ingress validation script to
  derive per-run nonce base`, then passed after the live validation derived
  nonce ranges from a per-run time/process seed.
- A later aggregate `make ci-tools` run failed at
  `scripts/runtime/test_run_input_mutation_contract_lane.sh` because the
  local-heavy aggregate path exceeded the `120s` input mutation default budget.
- `bash scripts/runtime/test_run_input_mutation_contract_lane.sh` first failed
  with `expected mutation lane default budget to cover aggregate local-heavy
  cargo compiles`, then passed after the local-only input mutation default
  budget was widened to `360s`.
- A later aggregate `make ci-tools` run failed at
  `scripts/runtime/test_run_lifecycle_property_contract_lane.sh` with
  `runtime lifecycle property contract lane exceeded runtime budget: 128s`.
- `bash scripts/runtime/test_run_lifecycle_property_contract_lane.sh` first
  failed with `expected lifecycle property contract lane default budget to
  cover aggregate local-heavy property tests`, then passed after the lifecycle
  property default budget was widened to `360s`.
- A later aggregate `make ci-tools` run failed at
  `scripts/runtime/test_run_concurrency_state_mutation_contract_lane.sh`.
- `bash scripts/runtime/test_run_concurrency_state_mutation_contract_lane.sh`
  first failed with `expected concurrency contract lane default budget to cover
  aggregate local-heavy mutation tests`, then passed after the concurrency
  mutation default budget was widened to `360s`.
- A clean aggregate `make ci-tools` rerun passed the earlier lifecycle timeout
  and then hung in `scripts/bridge/test_run_bridge_credentialed_contract_lane.sh`
  while a nested cross-chain outbound intent `cargo test` compiled against the
  shared `target/debug` tree at `0.0%` CPU for more than 11 minutes; the local
  gate was manually terminated with `SIGTERM`.
- `bash scripts/bridge/test_run_cross_chain_outbound_intent_contract_lane.sh`
  first failed with `expected outbound intent lane to expose an isolated target
  dir override`, then passed after the outbound intent contract lane used
  `KAMN_BRIDGE_OUTBOUND_INTENT_TARGET_DIR` and ran each Cargo test with
  `CARGO_TARGET_DIR="$bridge_outbound_intent_target_dir"`.
- `bash scripts/bridge/test_run_bridge_credentialed_contract_lane.sh` passes
  after the nested outbound intent lane isolation change.
- The next aggregate `make ci-tools` rerun passed the bridge credentialed and
  Telegram ingress lanes, then hung in
  `scripts/bridge/test_run_bridge_ingress_relay_contract_lane.sh` while
  `cargo test -p kamn-core --test bridge_ingress_relay_harness` compiled
  against the shared `target/debug` tree at `0.0%` CPU for more than two
  minutes; the local gate was manually terminated with `SIGTERM`.
- `bash scripts/bridge/test_run_bridge_ingress_relay_contract_lane.sh` first
  failed with `expected ingress relay lane to expose an isolated target dir
  override`, then passed after the bridge ingress relay lane used
  `KAMN_BRIDGE_INGRESS_RELAY_TARGET_DIR` and ran the Cargo test with
  `CARGO_TARGET_DIR="$bridge_ingress_relay_target_dir"`.
- The next aggregate `make ci-tools` rerun reached
  `scripts/kolme/test_run_local_kolme_fork_real_process_contract_lane.sh` and
  hung inside the nested process lifecycle readiness loop while
  `curl --silent --show-error --fail http://127.0.0.1:3000/healthz` had no
  per-attempt timeout; the local gate was manually terminated with `SIGTERM`.
- `bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh`
  first failed with `expected local fork process lifecycle readiness curl
  probes to set a connect timeout`, then passed after the readiness loop and
  planned readiness command added bounded `curl --connect-timeout` and
  `--max-time` arguments.
- The same lifecycle contract then failed with `expected local fork process
  lifecycle readiness loop to use a wall-clock deadline`, then passed after
  the readiness loop switched from a fixed attempt multiplier to a
  `readiness_deadline_epoch` bounded by `--startup-max-seconds`.
- The next aggregate `make ci-tools` rerun passed the readiness, runtime,
  message, governance, settlement, reputation, token, treasury, durable-guard,
  bridge credentialed, Telegram ingress, and bridge ingress relay sections,
  then hung in `scripts/bridge/test_run_bridge_outbound_quorum_contract_lane.sh`
  while `cargo test -p kamn-core --test bridge_outbound_quorum_execution`
  compiled against the shared `target/debug` tree at `0.0%` CPU for more than
  three minutes; the local gate was manually terminated with `SIGTERM`.
- `bash scripts/bridge/test_run_bridge_outbound_quorum_contract_lane.sh` first
  failed with `expected outbound quorum lane to expose an isolated target dir
  override`, then passed after the outbound quorum lane used
  `KAMN_BRIDGE_OUTBOUND_QUORUM_TARGET_DIR` and ran the Cargo test with
  `CARGO_TARGET_DIR="$bridge_outbound_quorum_target_dir"`.
- The next aggregate `make ci-tools` rerun passed through runtime snapshot,
  service API, message, governance, settlement, reputation, token, treasury,
  durable-guard, bridge credentialed, Telegram ingress, bridge ingress relay,
  outbound quorum, and bridge replay/redaction sections, then hung in
  `scripts/bridge/test_run_localhost_bridge_relay_demo_contract_lane.sh` while
  `cargo test -p kamn-core --test bridge_ingress_relay_harness` compiled
  against the shared `target/debug` tree at `0.0%` CPU for more than two
  minutes; the local gate was manually interrupted.
- `bash scripts/bridge/test_run_localhost_bridge_relay_demo_contract_lane.sh`
  first failed with `expected localhost bridge relay demo lane to expose an
  isolated target dir override`, then passed after the composite localhost
  bridge relay lane used `KAMN_LOCALHOST_BRIDGE_RELAY_DEMO_TARGET_DIR` and ran
  its signed demo plus bridge Cargo tests with
  `CARGO_TARGET_DIR="$localhost_bridge_relay_demo_target_dir"`.
- The next aggregate `make ci-tools` rerun passed through the fixed bridge
  ingress, outbound quorum, and localhost bridge relay sections, then hung in
  `scripts/bridge/test_run_bridge_adapter_conformance_contract_lane.sh` while
  `cargo test -p kamn-core --test bridge_adapter` compiled against the shared
  `target/debug` tree at `0.0%` CPU for more than seven minutes; the local gate
  was manually interrupted.
- `bash scripts/bridge/test_run_bridge_adapter_conformance_contract_lane.sh`
  first failed with `expected bridge adapter conformance lane to expose an
  isolated target dir override`, then passed after the conformance lane used
  `KAMN_BRIDGE_ADAPTER_CONFORMANCE_TARGET_DIR` and ran its Cargo tests with
  `CARGO_TARGET_DIR="$bridge_adapter_conformance_target_dir"`.
- The next aggregate `make ci-tools` rerun completed and ended with
  `All CI tool regression tests passed.`
- `cargo fmt --check` first failed on formatting drift in the live network
  smoke and live transport agent contract tests, then passed after `cargo fmt`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  first failed with `clippy::let_and_return` in
  `crates/kamn-sdk/tests/live_transport_agent/support/env_support.rs`, then
  passed after simplifying `bind_loopback_listener`; runtime: `68m 30s`.
- `make check` passed after rerunning `cargo fmt --check` and strict workspace
  clippy; runtime: `50m 44s`.
- `make pre-push` advanced through `make check` and much of `make ci-tools`,
  then failed at `scripts/message/test_run_message_lifecycle_contract_lane.sh`
  with only the parent wrapper line number because the test captured the fast
  lane under `set -e` and did not print the child output on failure.
- `bash scripts/message/test_run_message_lifecycle_contract_lane.sh` first
  failed with `expected message lifecycle wrapper test to capture fast-lane
  status explicitly`, then passed after the wrapper captured `fast_status`,
  printed `TMP_OUT` on failure, and preserved the success-marker assertion.
- The next `make pre-push` rerun reached
  `scripts/kolme/test_run_runtime_commit_contract_lane.sh` and stalled inside
  the prebuilt `kolme_runtime_commit_finality` test executable. The runtime
  commit runner only checked elapsed time after the subprocess returned, so a
  hung prebuilt test could wedge local pre-push without NO-GO evidence.
- `bash scripts/kolme/test_run_runtime_commit_contract_lane.sh` first failed
  with `expected Kolme runtime commit contract implementation to bound prebuilt
  test subprocesses`, then reported
  `Kolme runtime commit test timed out after 360s:
  kolme_runtime_commit_finality` after the runner applied a subprocess
  timeout.
- The same focused contract then failed with `expected Kolme runtime commit
  contract implementation to expose an isolated target dir override`, then
  passed after the runner used `KAMN_KOLME_RUNTIME_COMMIT_TARGET_DIR` /
  `CARGO_TARGET_DIR` for its Cargo prebuild; focused runtime: about `80s`.
- The next `make pre-push` rerun passed `make check`, `make ci-tools`, touched
  Rust size policy, and compiled full workspace tests, then wedged before the
  Rust harness started for
  `target/debug/deps/list_messages_service_contract-c640998ea838c8ce`.
  `sample` showed only `_dyld_start`, about `96K` footprint, and `0.0%` CPU.
  After `SIGTERM`, cargo advanced to
  `target/debug/deps/phase1_structure_contract-f1b300ece0604e2a` and showed
  the same pre-harness stall shape, so the aggregate run was terminated with
  code `143` instead of treating the local gate as green.
- `bash scripts/ci/test_local_pre_push_gate_policy.sh` first failed with
  `expected Makefile to include local pre-push configuration marker:
  PRE_PUSH_WORKSPACE_TARGET_DIR`, then passed after `make pre-push` gained
  `PRE_PUSH_WORKSPACE_TARGET_DIR`, `PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS`, and a
  workspace test leg that runs with
  `CARGO_TARGET_DIR="$(PRE_PUSH_WORKSPACE_TARGET_DIR)" timeout
  "$(PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS)"`.
- `make -n pre-push` now expands the workspace test leg to the isolated
  `target/local-pre-push-workspace` target directory and a `14400` second
  timeout while preserving
  `cargo test --workspace --locked --all-features --no-fail-fast`.
- `CARGO_TARGET_DIR=target/local-pre-push-workspace timeout 1800 bash
  scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p
  kamn-agent-lib --locked --all-features --test
  list_messages_service_contract --no-fail-fast -- --nocapture` passes:
  `1 passed`, compile runtime `2m 20s`.
- A later `make pre-push` rerun passed `make check`, `make ci-tools`, touched
  Rust size policy, and the isolated workspace test leg, then failed inside
  critical-path coverage because `cargo-llvm-cov` used the Homebrew rustc
  sysroot path
  `/opt/homebrew/Cellar/rust/1.90.0/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata`,
  which does not exist locally.
- `cargo test -p kamn-core --test critical_path_coverage_gate_contract
  critical_path_coverage_gate_resolves_llvm_tools_before_running_llvm_cov
  -- --exact` first failed with `missing llvm tool resolution marker:
  LLVM_COV`, then passed under
  `CARGO_TARGET_DIR=target/local-pre-push-workspace` after the coverage runner
  resolved executable `LLVM_COV` and `LLVM_PROFDATA` paths before invoking
  `cargo llvm-cov`.
- `bash -n scripts/ci/run_critical_path_coverage_gate.sh` passes.
- `bash scripts/ci/run_critical_path_coverage_gate.sh --threshold-file
  .ci/critical-path-coverage-thresholds.json --core-json
  /tmp/kamn-critical-path-core-coverage-pre-push-rerun.json --node-json
  /tmp/kamn-critical-path-node-coverage-pre-push-rerun.json --output-json
  /tmp/kamn-critical-path-coverage-policy-pre-push-rerun.json` now selects
  the rustup LLVM tool paths and passes with `status=ok`,
  `final_decision=GO`, `failed_targets=0`, and `missing_targets=0`.
- Final `make pre-push > /tmp/kamn-pre-push-final.log 2>&1` exited 0 on
  July 3, 2026 after running the full local gate sequence.
- Final pre-push `make check` passed strict clippy; the log records
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` and
  `Finished dev profile ... in 59m 45s`.
- Final pre-push `make ci-tools` passed with
  `All CI tool regression tests passed.`
- Final pre-push workspace tests used isolated
  `target/local-pre-push-workspace` and succeeded after 2 attempts. The first
  attempt failed only
  `integration_service_api_endpoint_websocket_streams_live_bridge_forwarded_event_after_upgrade`
  in `kamn-node`; the retry reran the full workspace leg and passed
  `local-pre-push-workspace-tests succeeded after 2 attempt(s)`.
- Final pre-push critical-path coverage passed with `status=ok`,
  `final_decision=GO`, `failed_targets=0`, `missing_targets=0`,
  `line_failures=0`, and `function_failures=0`. Report:
  `/tmp/kamn-critical-path-coverage-policy-pre-push.json`.
- Final pre-push critical-path mutation passed with `status=ok`,
  `final_decision=GO`, `tested_mutants=10`, `caught_mutants=10`,
  `missed_mutants=0`, `unviable_mutants=0`, and `timeout_mutants=0`. Report:
  `/tmp/kamn-critical-path-mutation-report-pre-push.json`.

## Shell-Surface DoD

- `shell_loc_delta_actual: +880`
- `rust_loc_delta_actual: +167024`
- `shell_to_rust_ratio_delta_actual: -0.179692`
- `shell_surface_ratio_target_status: improved`
