# CI Strategy

## Goal
Keep CI feedback fast and runner cost low while preserving confidence.

Detailed policy and script-surface budget rules live in `docs/ci/ci-cost-and-lane-framework.md`.

## Lane Split
- `ci-fast-gate` (PR required): minimal critical path for merge decisions.
- `ci-deep-validate` (nightly/manual): heavier suites outside PR hot path.

## Stage-1 Budget Targets
- Fast gate runtime: <= 8 minutes p50, <= 12 minutes p95.
- PR runner consumption: <= 25 total runner-minutes.
- Nightly deep validate: <= 120 minutes.

Versioned thresholds are defined in `.ci/ci-budget.env`.

## Fast Gate Behavior
`ci-fast-gate` calls `scripts/ci/select_targets.sh` to select execution scope from changed files:

- Docs-only changes: run markdown hygiene check only.
- Rust changes in specific crates/manifests: run targeted clippy/tests by manifest path.
- Core Rust metadata changes (`Cargo.toml`, `Cargo.lock`, toolchain, `.cargo`): run full workspace lane.
- CI/workflow changes without Rust source changes: run shell syntax checks and a smoke Rust lane when a Cargo project exists.
- Invariant-related changes (`invariants.rs`, `transaction.rs`, smoke/invariant harness tests, or harness scripts): run deterministic invariant harness in `fast` mode (single seed) after Rust tests.
- Runtime evaluator tests use direct unit-struct construction to avoid strict-clippy baseline noise (`Regression: #490`).

## Make and Selector Command-Surface Contract
Contributor entrypoint commands must remain stable and synchronized across `Makefile`, `README.md`, and selector routing policy:

- `make check`
- `make test`
- `make smoke-live-network`
- `make deep-live-network`
- `make demo`
- `make demo-localhost-transport`
- `make ci-tools`

Fast-gate command contract coverage is intentionally split:
- static command-surface parity (`scripts/ci/test_makefile_command_surface_contract.sh`)
- dry-run execution parity (`scripts/ci/test_makefile_execution_contract.sh`) via `make -n` target resolution for bounded `check/test/demo` targets
- Kolme missing-both coverage (`scripts/ci/test_kolme_command_surface_coverage_contract.sh`) ensures every `scripts/kolme/test_*.sh` appears in at least one CI command surface (`ci-fast-gate` or aggregate `scripts/ci/test_ci_tools.sh`).
- Kolme asymmetry split coverage (`scripts/ci/test_kolme_command_surface_asymmetry_contract.sh`) enforces the approved `fast_only` and `ci_tools_only` script sets from `.ci/kolme-command-surface-asymmetry-policy.json`.

Selector routing remains bounded through `scripts/ci/select_targets.sh`:

- `Makefile` changes map to runtime contract scope:
  - `run_runtime_snapshot_contract_tests=true`
  - `test_scope=runtime-contract`
- `.ci/kolme-command-surface-asymmetry-policy.json` changes map to CI contract scope:
  - `run_ci_tool_checks=true`
  - `test_scope=ci-doc-contract`
  - unknown/full fallback remains disabled for this path
- Localhost signed integration command changes map to dedicated scope:
  - `run_localhost_signed_integration_contract_lane_tests=true`
  - `test_scope=sdk-live-localhost-integration`
- Live transport replay/tamper command changes stay on the bounded localhost transport scope:
  - `run_localhost_signed_integration_contract_lane_tests=true`
  - `test_scope=sdk-live-localhost-integration`
  - `KAMN_SDK_REPLAY_TAMPER_CONTRACT_MAX_SECONDS=60`
- Live transport parity command changes map to parity scope:
  - `run_live_transport_parity_contract_tests=true`
  - `live_transport_parity_languages=rust,python,typescript`
- Core local-heavy Kolme helper/orchestration command changes map to dedicated local-heavy scope:
  - `run_kolme_local_heavy_contract_tests=true`
  - `test_scope=kolme-local-heavy-contract`
  - fast-gate commands:
    - `bash scripts/framework/test_assert_local_heavy_opt_in.sh`
    - `bash scripts/kolme/test_run_local_bootstrap_health_checks.sh`
    - `bash scripts/kolme/test_run_local_e2e_integration_lane.sh`
    - `bash scripts/kolme/test_run_local_heavy_validation_matrix.sh`
- Broader Kolme compatibility command changes continue to map to version-compatibility scope:
  - `run_kolme_version_compatibility_contract_tests=true`
  - `test_scope=kolme-version-contract`
  - command-surface tests stay on PR fast gate:
    - `bash scripts/kolme/test_run_fast_gate_native_api_parity_contract_lane.sh`
    - `bash scripts/kolme/test_generate_fork_compatibility_evidence.sh`
    - `bash scripts/kolme/test_check_fork_compatibility_policy.sh`
    - `bash scripts/kolme/test_run_runtime_commit_adapter_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_fork_sync_metadata_lane.sh`
    - `bash scripts/kolme/test_run_local_fork_smoke_evidence_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh`
    - `bash scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_api_probe_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_live_api_conformance_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_runtime_commit_live_lane.sh`
    - `bash scripts/kolme/test_run_local_native_api_parity_live_proof_contract_lane.sh`
    - `bash scripts/kolme/test_run_version_compatibility_contract_lane.sh`
  - tranche-1 manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_tranche1_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_snapshot_drift_contract_lane.sh`
      - `run_notifications_consumer_contract_lane.sh`
      - `run_block_fallback_reconciliation_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 60`
  - runtime+nonce manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_runtime_nonce_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_runtime_commit_adapter_contract_lane.sh`
      - `run_runtime_commit_replay_contract_lane.sh`
      - `run_nonce_broadcast_parity_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 120`
  - version+matrix manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_version_compatibility_contract_lane.sh`
      - `run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
      - `run_local_heavy_validation_matrix_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 120`
  - profile+self-test+portability manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_local_kolme_fork_profile_preflight_contract_lane.sh`
      - `run_local_kolme_fork_self_test_contract_lane.sh`
      - `run_local_kolme_fork_portability_preflight_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 120`
  - runtime+triadic+bootstrap+e2e manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_runtime_commit_contract_lane.sh`
      - `run_triadic_devnet_smoke_contract_lane.sh`
      - `run_local_bootstrap_health_checks_contract_lane.sh`
      - `run_local_e2e_integration_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 160`
  - bootstrap+conformance+runtime+process manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_local_kolme_fork_bootstrap_readiness_contract_lane.sh`
      - `run_local_kolme_live_api_conformance_contract_lane.sh`
      - `run_local_kamn_live_runtime_integration_contract_lane.sh`
      - `run_local_kolme_fork_process_lifecycle_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 160`
  - parity+demo+real-process manifest migration guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh`
    - enforces manifest-backed wrappers for:
      - `run_fast_gate_native_api_parity_contract_lane.sh`
      - `run_local_native_api_parity_live_proof_contract_lane.sh`
      - `run_local_signed_to_kolme_demo_contract_lane.sh`
      - `run_local_kolme_fork_checkout_bootstrap_contract_lane.sh`
      - `run_local_kolme_fork_real_process_contract_lane.sh`
    - enforces tranche shell-wrapper budget:
      - combined migrated wrapper shell LOC must remain `<= 200`
  - shared manifest-migration CI dispatcher guards stay on PR fast gate:
    - `bash scripts/ci/test_kolme_manifest_migration_contract_dispatch_wrapper_matrix.sh`
    - `bash scripts/ci/test_run_kolme_manifest_migration_contract_dispatch.sh`
    - enforces that each `test_kolme_*manifest_migration_contract.sh` wrapper dispatches through:
      - `scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh`
    - enforces shared migration group config contract:
      - `fixtures/ci/kolme_manifest_migration_contract_groups.json`
  - Kolme wrapper-inventory baseline guard stays on PR fast gate:
    - `bash scripts/ci/test_kolme_wrapper_inventory_baseline_contract.sh`
    - deterministic baseline artifact:
      - `fixtures/kolme_compatibility/wrapper_inventory_baseline.json`
    - deterministic generator/check commands:
      - `bash scripts/ci/generate_kolme_wrapper_inventory_baseline.sh --matrix-file fixtures/kolme_compatibility/lane_migration_matrix.json --output-json /tmp/kolme-wrapper-inventory-baseline.json`
      - `bash scripts/ci/check_kolme_wrapper_inventory_baseline.sh --matrix-file fixtures/kolme_compatibility/lane_migration_matrix.json --baseline-file fixtures/kolme_compatibility/wrapper_inventory_baseline.json --output-json /tmp/kolme-wrapper-inventory-delta.json`
    - emits lane-level and aggregate wrapper-count/shell-LOC deltas for migration trend governance.
    - Regression: #2117
  - shared dispatcher wrapper-matrix guard stays on PR fast gate:
    - `bash scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh`
    - enforces that all migrated `run_*contract_lane.sh` wrappers dispatch through:
      - `scripts/kolme/run_contract_lane_dispatch.sh`
    - validates manifest resolution fail-closed behavior for unknown wrapper keys.
    - Regression: #1827
  - additional Kolme contract checks stay covered by aggregate CI tools lane:
    - `bash scripts/kolme/test_run_nonce_broadcast_parity_contract_lane.sh`
    - `bash scripts/kolme/test_check_local_bootstrap_health_policy.sh`
    - `bash scripts/kolme/test_run_local_bootstrap_health_checks_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_profile_preflight_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_self_test_contract_lane.sh`
    - `bash scripts/kolme/test_run_local_kolme_fork_portability_preflight_contract_lane.sh`
  - nonce/broadcast parity matrix fast-lane budget stays bounded:
    - `KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60`
  - fast-gate native API parity lane remains bounded:
    - `bash scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh --output-json /tmp/kolme-fast-gate-native-api-parity-summary.json`
    - `python3 scripts/kolme/check_fast_gate_native_api_parity_policy.py --report-file /tmp/kolme-fast-gate-native-api-parity-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-fast-gate-native-api-parity-policy.json`
    - `KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120`
  - local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.
  - local-only fork sync/smoke run-mode commands remain excluded from ci-fast-gate.
    - `bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-sync-metadata-summary.json`
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --smoke-command "cargo test -p merkle-map --test version -- --exact load_from_zero_example" --max-seconds 120 --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json`
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json`
    - `bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json`
  - local Kolme API probe/smoke run-mode commands remain excluded from ci-fast-gate.
    - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 30 --output-json /tmp/kolme-local-api-probe-summary.json`
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --max-seconds 60 --output-json /tmp/kolme-local-api-smoke-summary.json`
  - local live API conformance harness run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 180 --probe-max-seconds 30 --native-max-seconds 120 --output-json /tmp/kolme-local-live-api-conformance-summary.json`
    - `python3 scripts/kolme/check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-api-conformance-policy.json`
    - `bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json`
  - local fork bootstrap/readiness run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 90 --probe-max-seconds 20 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json`
    - `python3 scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json`
    - `bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_contract_lane.sh --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json`
  - local KAMN live runtime integration run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds 210 --bootstrap-max-seconds 90 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
    - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
    - nested runtime step composes through `run_local_runtime_commit_live_finality_evidence_contract_lane.sh` and captures runtime policy artifacts.
    - integration summary must emit `ci_fast_gate_eligible=false` and `contracts.ci_fast_gate_scope=local-only`.
    - integration summary contracts must emit `runtime_profile` (`standard|real-node`) for deterministic operator/release evidence interpretation.
    - operator workflow reference: `Live Provider Operator Runbook (Issue #2114)` in `docs/planning/kolme-devnet-ops.md`.
    - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
    - `python3 scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json`
    - `bash scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json`
    - `bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
  - local fork process lifecycle integration run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --max-seconds 300 --startup-max-seconds 45 --integration-max-seconds 240 --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --integration-runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
    - process lifecycle integration command composition must include `--runtime-commit-live-policy-report` for nested integration evidence lineage.
    - process lifecycle summary/policy contracts must include deterministic rollback/recovery linkage markers (`--rollback-evidence-file`, `--recovery-evidence-file`).
    - `python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
    - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
  - local live-node validation bundle run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
    - `python3 scripts/kolme/check_local_live_node_validation_bundle_policy.py --report-file /tmp/kolme-local-live-node-validation-bundle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
    - `bash scripts/kolme/run_local_live_node_validation_bundle_contract_lane.sh --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json --policy-output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
    - bundle policy GO decisions require `ci_fast_gate_eligible=false` with `contracts.ci_fast_gate_scope=local-only` and complete nested evidence lineage.
  - local fork profile preflight run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode run --checkout-path /tmp/kolme_fork --max-seconds 45 --output-json /tmp/kolme-local-fork-profile-preflight-summary.json`
    - `python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file /tmp/kolme-local-fork-profile-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code profile_preflight_passed --output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
    - `bash scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-profile-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
  - local fork self-test run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --matrix-max-seconds 60 --matrix-cargo-profile portable --output-json /tmp/kolme-local-fork-self-test-summary.json`
    - `python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file /tmp/kolme-local-fork-self-test-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code fork_self_test_passed --output-json /tmp/kolme-local-fork-self-test-policy.json`
    - `bash scripts/kolme/run_local_kolme_fork_self_test_contract_lane.sh --output-json /tmp/kolme-local-fork-self-test-summary.json --policy-output-json /tmp/kolme-local-fork-self-test-policy.json`
  - local fork portability preflight run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_portability_preflight_lane.sh --mode run --checkout-path /tmp/kolme_fork --max-seconds 300 --output-json /tmp/kolme-local-fork-portability-preflight-summary.json`
    - `python3 scripts/kolme/check_local_kolme_fork_portability_preflight_policy.py --report-file /tmp/kolme-local-fork-portability-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code portability_preflight_passed --output-json /tmp/kolme-local-fork-portability-preflight-policy.json`
    - `bash scripts/kolme/run_local_kolme_fork_portability_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-portability-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-portability-preflight-policy.json`
  - local runtime-commit live run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --base-url http://127.0.0.1:3000 --provider-hint kolme-fork-local --max-seconds 90 --preflight-max-seconds 10 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
    - `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-runtime-commit-live-policy.json`
    - `bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json`
    - runtime submit/finality evidence markers (`submit_evidence_marker_present`, `finality_evidence_marker_present`) are required for GO decisions in run mode.
  - local native API parity live-proof run-mode commands remain excluded from ci-fast-gate.
    - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode run --nonce-command "curl --silent --show-error --fail http://127.0.0.1:3000/get-next-nonce?pubkey=test-key" --broadcast-command "curl --silent --show-error --fail --request PUT --data '{\"message\":\"native-parity\",\"signature\":\"sig\",\"recovery_id\":1}' http://127.0.0.1:3000/broadcast" --finality-command "curl --silent --show-error --fail http://127.0.0.1:3000/block/1" --max-seconds 180 --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json`
  - heavy execution keeps explicit opt-in: `KAMN_KOLME_LOCAL_HEAVY=1`
  - native parity fast/local command matrix remains synchronized across `README.md` and `docs/planning/kolme-devnet-ops.md`.
- Dashboard backend session/auth freshness command changes map to dashboard contract scope:
  - `run_dashboard_contract_tests=true`
  - `test_scope=frontend-contract`
- Dashboard stale/error budget command changes map to dashboard contract scope:
  - `run_dashboard_contract_tests=true`
  - `test_scope=frontend-contract`
- Frontend shell matrix command changes map to frontend scope:
  - `run_frontend_dashboard_tests=true`
  - `test_scope=frontend`
- Deployment SLO/rollback command changes map to deploy scope:
  - `run_deploy_preflight_tests=true`
  - `test_scope=deploy`
- Settlement evidence command changes map to escrow scope:
  - `run_settlement_reconciliation_contract_tests=true`
  - `test_scope=escrow-contract`
- Bridge adapter dry-run conformance command changes map to bridge scope:
  - `run_bridge_replay_harness=true`
  - `test_scope=bridge`
- Post-cutover SLO/alert evidence command changes map to canary scope:
  - `run_launch_canary_contract_tests=true`
  - `test_scope=canary-contract`
- Classification/redaction compliance command changes map to DSAR + channel scope:
  - `run_dsar_legal_hold_contract_tests=true`
  - `run_channel_lifecycle_contract_tests=true`
  - `test_scope=channel-contract`
- Governance lifecycle/rollback and quorum attestation command changes map to governance scope:
  - `run_governance_simulation_contract_tests=true`
  - `test_scope=governance-contract`

Required demo lane command contract:

- `bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json`
- `bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json`
- `bash scripts/sdk/check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-fast-report.json`
- `bash scripts/dashboard/run_backend_session_auth_freshness_contract_lane.sh --output-file /tmp/dashboard-backend-session-auth-freshness-contract-report.json`
- `bash scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh --output-file /tmp/dashboard-stale-error-contract-report.json`
- `bash scripts/frontend/run_dashboard_shell_determinism_matrix_contract_lane.sh --output-file /tmp/dashboard-shell-matrix-contract-report.json`
- `bash scripts/deploy/run_deployment_slo_rollback_contract_lane.sh --output-file /tmp/deployment-slo-rollback-contract-report.json`
- `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- `bash scripts/bridge/run_bridge_adapter_conformance_contract_lane.sh --output-json /tmp/bridge-adapter-conformance-contract-report.json`
- `bash scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh --output-json /tmp/kolme-fast-gate-native-api-parity-summary.json`
- `python3 scripts/kolme/check_fast_gate_native_api_parity_policy.py --report-file /tmp/kolme-fast-gate-native-api-parity-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-fast-gate-native-api-parity-policy.json`
- `bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode dry-run --output-json /tmp/kolme-local-bootstrap-summary.json`
- `python3 scripts/kolme/check_local_bootstrap_health_policy.py --report-file /tmp/kolme-local-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-bootstrap-policy.json`
- `bash scripts/kolme/run_local_bootstrap_health_checks_contract_lane.sh --output-json /tmp/kolme-local-bootstrap-summary.json --policy-output-json /tmp/kolme-local-bootstrap-policy.json`
- `bash scripts/kolme/run_local_e2e_integration_lane.sh --mode dry-run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- `python3 scripts/kolme/check_local_e2e_integration_policy.py --report-file /tmp/kolme-local-e2e-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-e2e-integration-policy.json`
- `bash scripts/kolme/run_local_e2e_integration_contract_lane.sh --output-json /tmp/kolme-local-e2e-integration-summary.json --policy-output-json /tmp/kolme-local-e2e-integration-policy.json`
- `bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- `python3 scripts/kolme/check_local_heavy_validation_matrix_policy.py --report-file /tmp/kolme-local-heavy-validation-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-heavy-validation-policy.json`
- `bash scripts/kolme/run_local_heavy_validation_matrix_contract_lane.sh --output-json /tmp/kolme-local-heavy-validation-summary.json --policy-output-json /tmp/kolme-local-heavy-validation-policy.json`
- `bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-api-conformance-summary.json`
- `bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json`
- `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- `bash scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-profile-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
- `bash scripts/kolme/run_local_kolme_fork_self_test_contract_lane.sh --output-json /tmp/kolme-local-fork-self-test-summary.json --policy-output-json /tmp/kolme-local-fork-self-test-policy.json`
- `bash scripts/kolme/run_local_kolme_fork_portability_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-portability-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-portability-preflight-policy.json`
- `bash scripts/canary/run_post_cutover_slo_contract_lane.sh`
- `bash scripts/compliance/run_classification_redaction_contract_lane.sh --output-file /tmp/classification-redaction-contract-report.json`
- `bash scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh --output-file /tmp/governance-lifecycle-rollback-contract-report.json`
- `bash scripts/governance/run_quorum_attestation_replay_contract_lane.sh --output-file /tmp/governance-quorum-attestation-replay-contract-report.json`

Regression policy:

- make-target and selector workflow drift remains fail-closed (`Regression: #900`).
- command-surface parity drift remains fail-closed (`Regression: #939`).
- dashboard stale/error selector/docs parity remains fail-closed (`Regression: #942`).
- frontend shell matrix selector/docs parity remains fail-closed (`Regression: #943`).
- dashboard backend session/auth freshness selector/docs parity remains fail-closed (`Regression: #941`).
- deployment slo/rollback selector/docs parity remains fail-closed (`Regression: #944`).
- settlement evidence selector/docs parity remains fail-closed (`Regression: #906`).
- bridge adapter conformance selector/docs parity remains fail-closed (`Regression: #907`).
- live transport replay/tamper selector/workflow/docs parity remains fail-closed (`Regression: #1386`).
- post-cutover slo/alert selector/docs parity remains fail-closed (`Regression: #913`).
- classification/redaction compliance selector/docs parity remains fail-closed (`Regression: #914`).
- governance lifecycle/rollback selector/docs parity remains fail-closed (`Regression: #910`).
- governance quorum attestation selector/docs parity remains fail-closed (`Regression: #911`).
- local-only heavy Kolme selector/workflow/docs parity remains fail-closed (`Regression: #1419`).
- aggregate CI-tools fork Rust matrix command-surface coverage remains fail-closed (`Regression: #1549`).
- local-only fork sync/smoke run-mode exclusion parity remains fail-closed (`Regression: #1431`).
- local-only heavy E2E policy and contract lane command-surface parity remains fail-closed (`Regression: #1682`).
- local-only heavy matrix policy and contract lane command-surface parity remains fail-closed (`Regression: #1687`).
- local fork profile preflight policy and contract lane command-surface parity remains fail-closed (`Regression: #1697`).
- local fork self-test policy and contract lane command-surface parity remains fail-closed (`Regression: #1702`).
- local fork portability preflight policy and contract lane command-surface parity remains fail-closed (`Regression: #1707`).
- local bootstrap health policy and contract lane command-surface parity remains fail-closed (`Regression: #1692`).
- local Kolme API probe/smoke run-mode exclusion parity remains fail-closed (`Regression: #1441`).
- local live API conformance harness run-mode exclusion parity remains fail-closed (`Regression: #1483`).
- local fork bootstrap/readiness run-mode exclusion parity remains fail-closed (`Regression: #1488`).
- local KAMN live runtime integration run-mode exclusion parity remains fail-closed (`Regression: #1489`).
- local KAMN live runtime integration runtime-step contract-lane composition and runtime policy artifact parity remain fail-closed (`Regression: #2101`).
- local KAMN live runtime integration runtime provider contract pass-through and nested runtime policy parity remain fail-closed (`Regression: #2112`).
- local KAMN live runtime integration local-only fast-gate exclusion summary markers remain fail-closed (`Regression: #2113`).
- live provider operator runbook command/checkpoint/troubleshooting marker parity remains fail-closed across docs and README references (`Regression: #2114`).
- local KAMN live runtime real-node profile policy+contract lane docs command-surface parity remains fail-closed (`Regression: #2139`).
- local live-node validation bundle contract lane and docs parity command surfaces remain fail-closed across devnet ops, CI strategy, and README (`Regression: #2134`).
- local fork process lifecycle integration run-mode exclusion parity remains fail-closed (`Regression: #1494`).
- local fork process lifecycle integration runtime policy report linkage to nested integration command composition and artifact lineage remains fail-closed (`Regression: #2104`).
- local fork process lifecycle rollback/recovery evidence linkage markers remain fail-closed in summary/policy/docs contracts (`Regression: #2107`).
- real-process wrapper lifecycle rollback/recovery evidence pass-through markers remain fail-closed across wrapper/lifecycle command composition and docs contracts (`Regression: #2109`).
- local runtime-commit live run-mode exclusion parity remains fail-closed (`Regression: #1451`).
- local runtime-commit live preflight health-probe and default live-provider ignored-test dispatch parity remains fail-closed (`Regression: #1829`).
- local runtime-commit live evidence policy marker parity remains fail-closed for missing `KolmeRuntimeCommitLiveProvider` command markers (`Regression: #2095`).
- local runtime-commit submit/finality evidence marker policy and contract lane command-surface parity remains fail-closed (`Regression: #2099`).
- local native API parity live-proof run-mode exclusion parity remains fail-closed (`Regression: #1467`).
- native parity fast/local command matrix docs parity remains fail-closed (`Regression: #1468`).
- local probe fork-info chain_version query and native parity broadcast method drift remains fail-closed (`Regression: #1482`).
- nonce/broadcast parity matrix selector/docs/runtime-budget drift remains fail-closed (`Regression: #1462`).
- fast-gate native Kolme API parity lane schema/routing/runtime-budget drift remains fail-closed (`Regression: #1466`).
- script-surface budget waiver schema/scope/expiry validation remains fail-closed when waiver files are present (`Regression: #1497`).
- script-surface duplicate-content policy excludes symlink dispatch wrappers and remains fail-closed for duplicated regular files (`Regression: #2090`).
- script-surface script-count/LOC metrics exclude `test_*.sh` harness scripts and remain fail-closed for non-test shell surfaces (`Regression: #2093`).
- Kolme command-surface missing-both coverage drift remains fail-closed (`Regression: #1561`).
- Kolme command-surface asymmetry split drift remains fail-closed (`Regression: #1565`).
- Kolme command-surface asymmetry policy-file schema drift remains fail-closed (`Regression: #1569`).
- Kolme tranche-1 manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1722`).
- Kolme runtime+nonce manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1763`).
- Kolme version+matrix manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1765`).
- Kolme profile+self-test+portability manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1767`).
- Kolme runtime+triadic+bootstrap+e2e manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1769`).
- Kolme bootstrap+conformance+runtime+process manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1771`).
- Kolme parity+demo+real-process manifest migration wrapper and shell-LOC budget drift remains fail-closed (`Regression: #1773`).
- Kolme shared manifest-migration CI dispatcher wrapper/config routing drift remains fail-closed (`Regression: #1833`).

## Budget Telemetry and Enforcement
Both lanes call `scripts/ci/evaluate_budget.sh` at the end of the run to:

- Compute elapsed runtime and approximate runner-minutes.
- Apply lane-specific warning/failure thresholds.
- Emit step-summary metrics for quick inspection.
- Upload JSON telemetry artifacts (`ci-budget-*.json`) for historical comparisons.

`ci-fast-gate` also generates and enforces a baseline delta report:

- `scripts/ci/generate_fast_gate_budget_delta_report.sh` emits baseline/current/variance metrics.
- `scripts/ci/check_fast_gate_budget_delta_threshold.sh` fails closed on unapproved regressions.
- `ci-budget-fast-gate-delta-*.json` artifacts are uploaded for auditability.

Policy:
- Warning at 90% of configured budget.
- Failure at 100% of configured budget for `ci-fast-gate` (merge-critical lane).
- Delta threshold overruns require a time-bounded waiver in `.ci/fast-gate-budget-delta-waiver.json` with tracked follow-up.

## Cache and Retry Telemetry
Telemetry includes:
- Rust cache hit status from `Swatinem/rust-cache` output.
- Whether bounded retry was used for test execution.

This data supports cache/parallel tuning and flaky-test burn-down without widening PR cost.

## Bounded Retry + Flaky Policy
- Tests run through `scripts/ci/run_with_retry.sh` with `max-attempts=2`.
- Retries are intentionally bounded to avoid hidden regressions.
- Flaky test quarantine inventory is tracked in `.ci/flaky-tests.txt`.
- Each quarantine entry must include owner, tracking issue, and expiry date.

## PR CI Impact Declaration
When CI-sensitive files are modified (`.github/workflows/*`, `scripts/ci/*`, `.ci/*`), PR description must explicitly declare CI impact.

Enforced by `scripts/ci/check_pr_ci_declaration.sh` in fast-gate.

## Script Regression Coverage
`ci-fast-gate` runs `scripts/ci/test_ci_tools.sh` with `KAMN_CI_TOOLS_FAST_MODE=true` to keep PR-critical CI tooling checks bounded and cost-effective, while local/deep lanes continue to run the full script.

Fast-mode CI tooling regression coverage includes:
- Budget evaluator (`test_evaluate_budget.sh`)
- Script duplication/surface budget checker (`test_check_script_duplication_budget.sh`)
- Retry helper (`test_run_with_retry.sh`)
- Invariant harness runner (`test_run_invariant_harness.sh`)
- Selector matrix runner with output-env isolation (`test_select_targets.sh`, `Regression: #463`)
- Flaky registry validator (`test_check_flaky_registry.sh`)
- Budget summarizer (`test_summarize_budget_artifacts.sh`)
- PR CI declaration checker (`test_check_pr_ci_declaration.sh`)
- Flaky report commenter (`test_post_flaky_report_comment.sh`)
- Flaky issue syncer (`test_sync_flaky_registry_issues.sh`)
- Workflow guard contracts (`test_workflow_retry_policy.sh`, `test_workflow_cache_policy.sh`, `test_workflow_scope_policy.sh`, `test_workflow_performance_policy.sh`)
- Rustdoc artifact lane contract (`test_run_kamn_core_rustdoc_artifact_contract_lane.sh`)
- Rustdoc artifact policy checker (`test_check_kamn_core_rustdoc_artifact_policy.sh`)
- Makefile execution contract checker (`test_makefile_execution_contract.sh`)
- Local fork portability preflight lane/policy/contract checks (`test_run_local_kolme_fork_portability_preflight_lane.sh`, `test_check_local_kolme_fork_portability_preflight_policy.sh`, `test_run_local_kolme_fork_portability_preflight_contract_lane.sh`)

## Reporting and Burn-down
- Weekly workflow `ci-flaky-registry` validates the quarantine registry and publishes a report artifact.
- Weekly workflow `ci-flaky-report-comment` posts an automated report comment to issue `#70`.
- Weekly workflow `ci-flaky-sync-issues` labels and updates tracking issues referenced in `.ci/flaky-tests.txt`.
- Use `scripts/ci/summarize_budget_artifacts.sh` on downloaded `ci-budget-*.json` artifacts to compute p50/p95 and cache/retry trends.
- Use `scripts/ci/download_and_summarize_budget.sh --repo <owner/repo>` to pull recent budget artifacts and produce a local trend summary.

## Deep Validation Behavior
`ci-deep-validate` runs full formatting, linting, and test suites on a nightly schedule and manually on demand.
It also runs deterministic invariant harness coverage in `deep` mode (bounded seed set) to keep invariant negative-path checks off the PR-critical lane while preserving repeatable coverage.

## Cost Controls
- Concurrency cancellation enabled on both workflows.
- Rust dependency/build cache enabled in Rust lanes.
- Expensive suites are not on the PR merge-critical path.
- PR template includes a mandatory CI-impact declaration for workflow/test-scope changes.

## Post-Billing Runbook
- Follow `docs/ci/post-billing-closeout.md` to close #68/#70 once hosted workflows are available.
