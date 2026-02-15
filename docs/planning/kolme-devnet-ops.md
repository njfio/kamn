# Kolme Triadic Devnet Operability Plan (Issues #784, #785, #787, #788, #1405, #1417, #1418, #1501)

This plan defines the deterministic, low-cost local smoke contract for triadic
runtime roles (processor/listener/approver) and its CI-compatible validation.

The live backend contract inventory for `njfio/kolme_fork` is tracked in:
- `docs/research/kolme-fork-api-contract-inventory.md`

## Scope

- One-command triadic devnet smoke orchestration.
- Deterministic marker validation from fixture contract.
- PR-safe runtime budget guard for smoke lane cost control.

## Composed Full-Stack E2E Lane (Issue #3420)

- Composed runtime lane command:
  - `bash scripts/runtime/validate_local_full_stack_integration_live.sh --mode dry-run --output-json /tmp/local-full-stack-integration-summary.json`
- Local-only composed run-mode command:
  - `KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN=1 bash scripts/runtime/validate_local_full_stack_integration_live.sh --mode run --ci-fast-gate FAIL --kolme-checkout-path /tmp/kolme_fork --kolme-expected-remote-url https://github.com/njfio/kolme_fork.git --kolme-expected-ref refs/heads/main --kolme-base-url http://127.0.0.1:3000 --kolme-fork-chain-version v0.15.2 --output-json /tmp/local-full-stack-integration-summary.json`
- Policy checker command:
  - `bash scripts/runtime/check_local_full_stack_integration_live_policy.sh --report-file /tmp/local-full-stack-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-stack-integration-policy.json`
- Contract lane command:
  - `bash scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh --output-json /tmp/local-full-stack-integration-contract-lane-report.json --policy-output-json /tmp/local-full-stack-integration-policy.json`
- Composition contract:
  - run-mode composes:
    - `scripts/runtime/validate_full_io_scenario_matrix_live.sh`
    - `scripts/runtime/validate_local_full_runtime_live.sh`
    - `scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh`
  - nested Kolme summary/policy evidence is fail-closed for:
    - signer provenance markers
    - runtime commit submission markers
    - runtime commit finality markers
    - provider contract marker `KolmeRuntimeCommitLiveProvider`
    - local checkout/remote/ref/base-url/fork-chain prerequisite markers
    - local-only enforcement and nested run-mode policy reason-code marker `live_runtime_integration_passed`
- Deterministic tamper reason:
  - `local_full_stack_integration_policy_runtime_commit_finality_status_mismatch`
- Release go/no-go linkage:
  - `scripts/runtime/release_evidence_manifest.json` includes required artifact id `local_full_stack_integration`.
  - release gate runner consumes `validate_local_full_stack_integration_live_contract_lane.sh` and fails closed on missing/tampered evidence linkage.
- Architecture boundary reference:
  - `docs/architecture/kolme-live-integration.md`

## Lane Migration Matrix (Issue #1721)

- Canonical prioritized lane migration matrix fixture:
  - `fixtures/kolme_compatibility/lane_migration_matrix.json`
- Matrix schema version:
  - `kamn.kolme.lane-migration-matrix.v1`
- Fail-closed matrix policy checker:
  - `python3 scripts/kolme/check_lane_migration_matrix_policy.py --matrix-file fixtures/kolme_compatibility/lane_migration_matrix.json`
- Matrix contract test:
  - `bash scripts/kolme/test_check_lane_migration_matrix_policy.sh`
- Required waiver-critical lane identifiers:
  - `kolme.version.compatibility`
  - `kolme.runtime.commit.adapter`
  - `kolme.runtime.commit.replay`
  - `kolme.notifications.consumer`
  - `kolme.block.fallback.reconciliation`
  - `kolme.nonce.broadcast.parity`
  - `kolme.local.fork.rust_matrix`
  - `kolme.local.kamn.live_runtime_integration`
  - `kolme.local.heavy.validation_matrix`

## Fallback Marker Retirement Matrix (Issue #2526)

- Canonical fallback marker classification fixture:
  - `fixtures/kolme_compatibility/fallback_signer_marker_matrix.json`
- Matrix schema version:
  - `kamn.kolme.fallback-signer-marker-matrix.v1`
- Fail-closed matrix policy checker:
  - `python3 scripts/kolme/check_fallback_signer_marker_matrix_policy.py --matrix-file fixtures/kolme_compatibility/fallback_signer_marker_matrix.json`
- Matrix contract test:
  - `bash scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh`
- Classification contract:
  - `keep` markers remain active fail-closed controls.
  - `deprecate` markers remain compatibility-visible while retirement sequencing is active.
  - `remove-target` markers track surfaces scheduled for removal in later tranches.

## Shared Wrapper Dispatcher Tranche (Issue #1827)

- Shared Kolme contract-lane dispatcher:
  - `scripts/kolme/run_contract_lane_dispatch.sh`
- Compatibility wrapper shape:
  - all manifest-only `scripts/kolme/run_*contract_lane.sh` wrappers are symlinks to the shared dispatcher.
- Script-surface duplicate-content policy alignment:
  - symlinked dispatcher wrappers are excluded from `duplicate_content` budget checks; duplicate-content enforcement remains fail-closed for regular files.
  - `test_*.sh` harness scripts are excluded from script-surface `script_count`/`shell_line_total` metrics so budget gates track operational shell command surface only.
  - current script-surface budgets pass without `.ci/script-surface-budget-waiver.json`; waiver dependency is retired on mainline.
- Dispatcher matrix guard:
  - `bash scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh`
- CI contract surface:
  - dispatcher matrix guard runs in both:
    - `.github/workflows/ci-fast-gate.yml`
    - `scripts/ci/test_ci_tools.sh`
- Regression marker:
  - `Regression: #1827`
  - `Regression: #2090`
  - `Regression: #2093`

## Wave-10 Wrapper Baseline + Trend Governance (Issue #2281)

- Wave-10 wrapper-family matrix fixture:
  - `fixtures/ci/kolme_wave10_wrapper_family_matrix.json`
- Wave-10 baseline fixture:
  - `fixtures/ci/kolme_wave10_wrapper_family_baseline.json`
- Wave-10 trend threshold fixture:
  - `fixtures/ci/kolme_wave10_wrapper_family_trend_thresholds.json`
- Baseline generator/check commands:
  - `bash scripts/ci/generate_kolme_wrapper_inventory_baseline.sh --matrix-file fixtures/ci/kolme_wave10_wrapper_family_matrix.json --output-json /tmp/kolme-wave10-wrapper-family-baseline.json`
  - `bash scripts/ci/check_kolme_wrapper_inventory_baseline.sh --matrix-file fixtures/ci/kolme_wave10_wrapper_family_matrix.json --baseline-file fixtures/ci/kolme_wave10_wrapper_family_baseline.json --output-json /tmp/kolme-wave10-wrapper-family-delta.json`
- Trend checker command:
  - `bash scripts/ci/check_kolme_wave10_wrapper_family_budget_trend.sh --matrix-file fixtures/ci/kolme_wave10_wrapper_family_matrix.json --baseline-file fixtures/ci/kolme_wave10_wrapper_family_baseline.json --output-json /tmp/kolme-wave10-wrapper-family-trend-report.json`
- Wave-10 lane family coverage:
  - `scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh`
- Fail-closed reason-code surface:
  - `wrapper_count_delta_threshold_exceeded`
  - `total_shell_loc_delta_threshold_exceeded`
  - `lane_shell_loc_increase_violation`
  - `unexpected_new_lanes_in_current_inventory`

## Shared Manifest-Migration CI Dispatcher (Issue #1833)

- Shared migration CI dispatcher:
  - `scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh`
- Shared dispatcher validator implementation:
  - `scripts/ci/kolme_manifest_migration_contract.py`
- Shared migration-group config contract:
  - `fixtures/ci/kolme_manifest_migration_contract_groups.json`
- Thin-wrapper matrix guard:
  - `bash scripts/ci/test_kolme_manifest_migration_contract_dispatch_wrapper_matrix.sh`
- Dispatcher behavior guard:
  - `bash scripts/ci/test_run_kolme_manifest_migration_contract_dispatch.sh`
- Compatibility wrapper shape:
  - each `scripts/ci/test_kolme_*manifest_migration_contract.sh` entrypoint is now a thin wrapper that delegates to the shared dispatcher with a fixed `--group`.
- Regression marker:
  - `Regression: #1833`

## Tranche-1 Manifest Migration (Issue #1722)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_tranche1_manifest_migration_contract.sh`
- Execution parity contract (wrapper entrypoint vs direct manifest dispatch):
  - `bash scripts/ci/test_kolme_tranche1_dispatch_execution_parity_contract.sh`
  - compares normalized execution output for each tranche lane between:
    - `scripts/kolme/run_*_contract_lane.sh`
    - `scripts/framework/run_manifest_lane.sh --manifest ... --phase contract`
  - stays in aggregate `scripts/ci/test_ci_tools.sh` (not PR fast gate) to preserve bounded fast-gate runtime cost.
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_snapshot_drift_contract_lane.sh`
  - `scripts/kolme/run_notifications_consumer_contract_lane.sh`
  - `scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_snapshot_drift_contract_lane.json`
  - `scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json`
  - `scripts/framework/manifests/kolme_block_fallback_reconciliation_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/snapshot_drift_contract_lane.py`
  - `scripts/kolme/contracts/notifications_consumer_contract_lane.py`
  - `scripts/kolme/contracts/block_fallback_reconciliation_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for the tranche remains `<= 60`.

## Runtime+Nonce Manifest Migration (Issue #1763)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_runtime_nonce_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_runtime_commit_adapter_contract_lane.sh`
  - `scripts/kolme/run_runtime_commit_replay_contract_lane.sh`
  - `scripts/kolme/run_nonce_broadcast_parity_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json`
  - `scripts/framework/manifests/kolme_runtime_commit_replay_contract_lane.json`
  - `scripts/framework/manifests/kolme_nonce_broadcast_parity_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/runtime_commit_adapter_contract_lane.py`
  - `scripts/kolme/contracts/runtime_commit_replay_contract_lane.py`
  - `scripts/kolme/contracts/nonce_broadcast_parity_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this runtime/nonce tranche remains `<= 120`.

## Version+Matrix Manifest Migration (Issue #1765)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_version_compatibility_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
  - `scripts/kolme/run_local_heavy_validation_matrix_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_version_compatibility_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_fork_rust_test_matrix_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_heavy_validation_matrix_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/version_compatibility_contract_lane.py`
  - `scripts/kolme/contracts/local_fork_rust_test_matrix_contract_lane.py`
  - `scripts/kolme/contracts/local_heavy_validation_matrix_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this version/matrix tranche remains `<= 120`.

## Profile+SelfTest+Portability Manifest Migration (Issue #1767)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_self_test_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_portability_preflight_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_local_fork_profile_preflight_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_fork_self_test_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_fork_portability_preflight_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/local_fork_profile_preflight_contract_lane.py`
  - `scripts/kolme/contracts/local_fork_self_test_contract_lane.py`
  - `scripts/kolme/contracts/local_fork_portability_preflight_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this profile/self-test/portability tranche remains `<= 120`.

## Runtime+Triadic+Bootstrap+E2E Manifest Migration (Issue #1769)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_runtime_commit_contract_lane.sh`
  - `scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh`
  - `scripts/kolme/run_local_bootstrap_health_checks_contract_lane.sh`
  - `scripts/kolme/run_local_e2e_integration_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_runtime_commit_contract_lane.json`
  - `scripts/framework/manifests/kolme_triadic_devnet_smoke_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_bootstrap_health_checks_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_e2e_integration_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/runtime_commit_contract_lane.py`
  - `scripts/kolme/contracts/triadic_devnet_smoke_contract_lane.py`
  - `scripts/kolme/contracts/local_bootstrap_health_checks_contract_lane.py`
  - `scripts/kolme/contracts/local_e2e_integration_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this runtime/triadic/bootstrap/e2e tranche remains `<= 160`.

## Bootstrap+Conformance+Runtime+Process Manifest Migration (Issue #1771)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_local_kolme_fork_bootstrap_readiness_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh`
  - `scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/local_kolme_fork_bootstrap_readiness_contract_lane.py`
  - `scripts/kolme/contracts/local_kolme_live_api_conformance_contract_lane.py`
  - `scripts/kolme/contracts/local_kamn_live_runtime_integration_contract_lane.py`
  - `scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this bootstrap/conformance/runtime/process tranche remains `<= 160`.

## Parity+Demo+Real-Process Manifest Migration (Issue #1773)

- Migration guard contract:
  - `bash scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh`
- Migrated manifest-backed wrappers:
  - `scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh`
  - `scripts/kolme/run_local_native_api_parity_live_proof_contract_lane.sh`
  - `scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh`
- Manifest files:
  - `scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_native_api_parity_live_proof_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json`
  - `scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json`
- Python contract lane implementations:
  - `scripts/kolme/contracts/fast_gate_native_api_parity_contract_lane.py`
  - `scripts/kolme/contracts/local_native_api_parity_live_proof_contract_lane.py`
  - `scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py`
  - `scripts/kolme/contracts/local_kolme_fork_checkout_bootstrap_contract_lane.py`
  - `scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py`
- Shell-surface budget contract:
  - Combined wrapper shell LOC for this parity/demo/real-process tranche remains `<= 200`.

## Contract Commands

- Run triadic smoke orchestration:
  - `bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file /tmp/triadic-devnet-markers.txt`
- Validate observed markers:
  - `python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file /tmp/triadic-devnet-markers.txt --output-json /tmp/triadic-devnet-report.json`
- Run budgeted contract lane:
  - `bash scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh --output-json /tmp/triadic-devnet-report.json`

## Deterministic Marker Contract

- Fixture file:
  - `fixtures/kolme_compatibility/devnet_smoke_markers.json`
- Required markers:
  - `marker_startup=ok`
  - `marker_tx_progression=ok`
  - `marker_block_commit=ok`
  - `marker_teardown=ok`
  - `status=pass`

## Runtime and Cost Policy

- PR contract lane budget:
  - `run_triadic_devnet_smoke.sh` and `run_triadic_devnet_smoke_contract_lane.sh` enforce a 180-second ceiling.
- Bounded runtime calls:
  - smoke runner executes only targeted triadic role smoke tests to avoid full-suite costs.
- CI compatibility:
  - lane is non-interactive and emits machine-readable validation report output.

## Runtime Commit Adapter Replay/Finality Fast Lane (Issue #980)

- Adapter replay/finality contract lane:
  - `bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh`
- Reason-code checks:
  - `receipt_provider_mismatch`
  - `receipt_not_final`
- Cost policy:
  - lane enforces a 60-second fast-gate budget and runs only targeted adapter/replay checks.

## Runtime Commit Block Fallback Reconciliation Fast Lane (Issue #1464)

- Block fallback reconciliation contract lane:
  - `bash scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh`
- Targeted rust fallback reconciliation test:
  - `cargo test -p kamn-core --test kolme_runtime_commit_block_fallback`
- Cost policy:
  - lane enforces a 75-second fast-gate budget via `KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS`.

## Deterministic Local Fork Sync Metadata Lane (Issue #1429)

- Local fork metadata sync runner:
  - `bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-sync-metadata-summary.json`
- Local fork metadata validation:
  - `bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-sync-metadata-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_fork_sync_metadata_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_fork_sync_metadata_lane.json`
- Summary schema:
  - `kamn.kolme.local-fork-sync-metadata-summary.v1`
- Deterministic checks include:
  - checkout path exists
  - checkout is a git work tree
  - `origin` remote URL matches expected fork repository
  - symbolic HEAD ref matches expected ref
  - HEAD commit is non-empty
  - checkout dirty-state guard remains fail-closed unless explicit `--allow-dirty` is set

## Bounded Local Fork Smoke Evidence Lane (Issue #1430)

- Local fork smoke evidence runner:
  - `bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json`
- Explicit local-only smoke execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --smoke-command "cargo test -p merkle-map --test version -- --exact load_from_zero_example" --max-seconds 120 --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_fork_smoke_evidence_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_fork_smoke_evidence_lane.json`
- Summary schema:
  - `kamn.kolme.local-fork-smoke-evidence-summary.v1`
- Deterministic checkpoints include:
  - `run_local_fork_sync_metadata_lane.sh` metadata validation
  - bounded smoke command execution with timeout budget guard
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - shared opt-in enforcement helper: `scripts/framework/assert_local_heavy_opt_in.sh`.
  - smoke command timeout/exceeded budget is reported as `fork_smoke_command_timeout`.

## Local-Only Fork Rust Test Matrix Lane (Issue #1537)

- Local fork Rust test matrix runner:
  - `bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json`
- Explicit local-only Rust test matrix execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --cargo-profile portable --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_rust_test_matrix_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_rust_test_matrix_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py --report-file /tmp/kolme-local-fork-rust-test-matrix-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-rust-test-matrix-summary.v1`
- Deterministic checkpoints include:
  - `run_local_fork_sync_metadata_lane.sh` metadata validation prior to Rust command execution.
  - portable cargo profile support (`--cargo-profile portable`) rewrites cargo invocations with `RUSTFLAGS=''` for linker-portable local execution.
  - bounded per-command timeout guard with deterministic pass/fail reason codes.
  - per-command stdout/stderr artifact capture for audit review.
  - evidence bundle marker contract: `evidence_bundle_schema_version=kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1`.
  - evidence bundle payload contract: `evidence_bundle` includes `schema_version`, `summary_schema_version`, `status`, `reason_code`, `budget_status`, `command_count`, and `artifact_paths`.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - run mode remains local/manual and is excluded from PR fast-gate workflow routing.
  - Regression: #1541
  - Regression: #2329

## Deterministic Local Kolme API Probe Lane (Issue #1439)

- Local API probe runner:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-api-probe-summary.json`
- Active local API probe:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 30 --output-json /tmp/kolme-local-api-probe-summary.json`
- Summary schema:
  - `kamn.kolme.local-api-probe-summary.v1`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_api_probe_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_api_probe_lane.json`
- Deterministic checks include:
  - `GET /healthz` response body matches expected health marker (`Healthy!` by default).
  - `GET /fork-info?chain_version=<version>` returns valid JSON object with integer `first_block` and `last_block`.
- Cost policy:
  - run mode enforces a deterministic runtime budget ceiling via `--max-seconds`.

## Bounded Local-Only Kolme API Smoke Lane (Issue #1440)

- Local API smoke runner:
  - `bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --output-json /tmp/kolme-local-api-smoke-summary.json`
- Explicit local-only API smoke execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --max-seconds 60 --output-json /tmp/kolme-local-api-smoke-summary.json`
- Summary schema:
  - `kamn.kolme.local-api-smoke-summary.v1`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_api_smoke_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_api_smoke_lane.json`
- Deterministic checkpoints include:
  - `run_local_kolme_api_probe_lane.sh` prerequisite run-mode verification.
  - bounded smoke command execution with timeout budget guard.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - smoke command timeout/exceeded budget is reported as `smoke_command_timeout`.

## Local-Only Live Kolme API Conformance Harness (Issue #1483)

- Local live API conformance harness runner:
  - `bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-api-conformance-summary.json`
- Explicit local-only live conformance execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 180 --probe-max-seconds 30 --native-max-seconds 120 --output-json /tmp/kolme-local-live-api-conformance-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-api-conformance-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json`
- Matrix fixture:
  - `fixtures/kolme_commit/local_live_api_conformance_matrix.json`
- Summary schema:
  - `kamn.kolme.local-live-api-conformance-summary.v1`
- Deterministic checkpoints include:
  - `run_local_kolme_api_probe_lane.sh` run-mode verification for `GET /healthz` and `GET /fork-info?chain_version=<version>`.
  - `run_local_native_api_parity_live_proof_lane.sh` run-mode verification for `GET /get-next-nonce` and `PUT /broadcast`.
  - deterministic fail-closed reason codes for missing opt-in, probe/native prerequisite failures, and runtime budget overruns.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 180 seconds.
  - local live conformance run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Kolme Fork Bootstrap/Readiness Contract Lane (Issue #1488)

- Local fork bootstrap/readiness runner:
  - `bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json`
- Explicit local-only bootstrap/readiness execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 90 --probe-max-seconds 20 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_bootstrap_readiness_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_contract_lane.sh --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-bootstrap-readiness-summary.v1`
- Deterministic checkpoints include:
  - `run_local_fork_sync_metadata_lane.sh` run-mode metadata verification for pinned checkout provenance.
  - `run_local_kolme_api_probe_lane.sh` run-mode verification for `GET /healthz` and `GET /fork-info?chain_version=<version>`.
  - deterministic fail-closed reason codes for missing opt-in, sync/probe prerequisite failures, and runtime budget overruns.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 90 seconds for local reproducibility.
  - local bootstrap/readiness run-mode execution remains excluded from PR fast-gate workflow routing.

## Local KAMN Live Runtime Integration Lane (Issue #1489)

- Local KAMN live runtime integration runner:
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Explicit local-only live runtime integration execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds 210 --bootstrap-max-seconds 90 --localhost-signed-max-seconds 45 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Real-node profile contract marker (local endpoint/operator validation):
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --runtime-signer-profile ops-secondary --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
  - `KAMN_KOLME_LIVE_SIGNER_KEY_REF=secure:aws-kms:role-operator/key-live-ops-primary KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX=0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --runtime-signer-key-source managed-external --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Optional runtime finality pass-through to nested live runner:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-finality-command "printf 'finality=final\n'" --runtime-commit-finality-max-seconds 15 --runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Real-node profile policy checker command:
  - `python3 scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --require-non-synthetic-run-evidence --output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json`
- Strict-marker GO/NO-GO proofs:
  - GO proof uses the command above and requires deterministic markers:
    - `runtime_commit_command_profile=real-node-non-synthetic-v1`
    - `runtime_commit_policy_command_profile=real-node-non-synthetic-v1`
    - `runtime_commit_command_profile_version=v1`
    - `runtime_signer_profile_selector_env=KAMN_KOLME_LIVE_SIGNER_PROFILE`
    - `runtime_signer_profile=ops-primary`
    - `runtime_signer_previous_profile=ops-primary`
    - `runtime_signer_failover_active=false`
    - `runtime_signer_rotation_epoch=1`
    - `runtime_signer_previous_rotation_epoch=1`
    - `runtime_signer_key_source_contract_version=v1`
    - `runtime_signer_key_source=env-local`
    - `runtime_signing_profile=kolme-fork-secp256k1-v1`
    - `runtime_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX`
    - `runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF`
    - `runtime_signer_fallback_guard_contract_version=v2`
    - `runtime_signer_fallback_guard_mode=reject_if_present`
    - `runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF`
    - `runtime_signer_fallback_private_key_present=false`
    - `runtime_signer_raw_private_key_present=false`
  - GO proof also supports deterministic secondary signer markers:
    - `runtime_signer_profile=ops-secondary`
    - `runtime_signer_previous_profile=ops-secondary`
    - `runtime_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY`
  - NO-GO marker-drift proof must surface `runtime_commit_command_profile_mismatch` when profile marker contracts drift from `real-node-non-synthetic-v1`.
  - NO-GO signer-profile drift proof must surface `runtime_signer_profile_mismatch` when summary signer profile markers drift from `ops-primary`.
  - NO-GO failover proof must surface `runtime_signer_failover_profile_unchanged` when failover is active but profile does not rotate.
  - NO-GO stale-rotation proof must surface `runtime_signer_rotation_epoch_stale` when failover rotation epoch is not strictly increasing.
  - NO-GO split-brain proof must surface `runtime_commit_signer_profile_split_brain_detected` when runtime command composition includes conflicting signer profile selectors.
  - NO-GO key-source/profile matrix proof must surface `runtime_signer_key_source_profile_pair_disallowed` when signer profile/key-source pair is outside the strict allowlist.
  - NO-GO signer-key-env drift proof must surface `runtime_signer_private_key_env_mismatch` when signer key env marker drifts from `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX`.
  - NO-GO signer key-source command marker proof must surface `runtime_commit_signer_key_source_marker_missing` when `runtime_commit_command` omits `KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=<env-local|managed-external>`.
  - NO-GO fallback signer key command marker proof must surface `runtime_commit_fallback_private_key_command_marker_detected` when `runtime_commit_command` includes `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK=...`.
  - NO-GO managed-external key-reference command marker proof must surface `runtime_commit_managed_external_signer_key_reference_marker_missing` when managed-external command composition omits `KAMN_KOLME_LIVE_SIGNER_KEY_REF=...`.
  - NO-GO managed-external signer public-key command marker proof must surface `runtime_commit_managed_external_signer_public_key_marker_missing` when managed-external command composition omits profile-specific signer public-key markers (`KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX` / `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY`).
  - NO-GO managed-external private key command marker proof must surface `runtime_commit_managed_external_private_key_command_marker_detected` when managed-external command composition includes `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=...`.
  - NO-GO runtime-signing-profile drift proof must surface `runtime_signing_profile_mismatch` when summary runtime signing profile markers drift from `kolme-fork-secp256k1-v1`.
  - NO-GO runtime-signing-profile contract drift proof must surface `runtime_signing_profile_contract_mismatch` when contracts runtime signing profile markers drift from `kolme-fork-secp256k1-v1`.
  - NO-GO fallback signer key proof must surface `runtime_signer_fallback_private_key_present_violation` when `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` is present.
  - NO-GO managed-external raw signer key proof must surface `runtime_signer_managed_external_raw_private_key_present_violation` when `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX` is present while `runtime_signer_key_source=managed-external`.
  - NO-GO synthetic-command regression proof must surface `runtime_commit_non_synthetic_submit_probe_missing` when `runtime_commit_command` omits `integration_kolme_fork_live_node_submit_reaches_endpoint`.
  - NO-GO synthetic-command regression proof must surface `runtime_commit_real_signing_profile_marker_missing` when `runtime_commit_command` omits `KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1`.
  - NO-GO synthetic-command regression proof must surface `runtime_commit_signer_profile_marker_missing` when `runtime_commit_command` omits `KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary`.
  - NO-GO synthetic-command regression proof must surface `runtime_commit_signer_profile_marker_missing` when `runtime_commit_command` omits `KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary`.
  - NO-GO synthetic-command regression proof must surface `runtime_commit_signer_profile_split_brain_detected` when `runtime_commit_command` includes conflicting `KAMN_KOLME_LIVE_SIGNER_PROFILE` selectors in one command composition.
- Real-node profile contract lane command:
  - `bash scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json`
  - `bash scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh --runtime-signer-profile ops-secondary --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Summary schema:
  - `kamn.kolme.local-kamn-live-runtime-integration-summary.v1`
- Deterministic checkpoints include:
  - wrapper routing remains manifest-backed via `scripts/kolme/run_lane_dispatch.sh` resolving `scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_lane.json`.
  - `run_local_kolme_fork_bootstrap_readiness_lane.sh` run-mode validation for pinned checkout provenance and API readiness.
  - `run_localhost_signed_integration_contract_lane.sh` run-mode validation of signed message admission/replay guards before Kolme runtime commit execution.
  - `run_local_kolme_live_api_conformance_harness.sh` run-mode validation for health/query/nonce/broadcast command contracts.
  - `run_local_runtime_commit_live_finality_evidence_contract_lane.sh` is composed as the default runtime-commit endpoint step (no raw curl fallback by default).
  - optional runtime finality pass-through (`--runtime-commit-finality-command`, `--runtime-commit-finality-max-seconds`, `--runtime-commit-finality-output-file`) and `--runtime-commit-live-policy-report` are wired through to nested runtime finality evidence contract composition.
  - runtime provider contract marker (`--runtime-provider-client-contract`) remains explicit and fail-closed for `KolmeRuntimeCommitLiveProvider`.
  - runtime integration profile marker (`--runtime-profile standard|real-node`) remains explicit and is emitted into summary + contracts fields for release-decision audits.
  - deterministic runtime evidence profile markers are emitted in summary fields:
    - `runtime_commit_command_profile`
    - `runtime_commit_policy_command_profile`
    - `runtime_commit_command_profile_version`
    - `runtime_signer_profile_selector_env`
    - `runtime_signer_profile`
    - `runtime_signer_previous_profile`
    - `runtime_signer_failover_active`
    - `runtime_signer_rotation_epoch`
    - `runtime_signer_previous_rotation_epoch`
    - `runtime_signer_key_source_contract_version`
    - `runtime_signer_key_source`
    - `runtime_signer_private_key_env`
    - `runtime_signer_key_reference_env`
    - `runtime_signer_fallback_guard_contract_version`
    - `runtime_signer_fallback_guard_mode`
    - `runtime_signer_fallback_private_key_present`
    - `runtime_signer_raw_private_key_present`
  - real-node profile requires `runtime_commit_command_profile=real-node-non-synthetic-v1`, `runtime_commit_policy_command_profile=real-node-non-synthetic-v1`, and `runtime_commit_command_profile_version=v1`; real-node checker fails closed on marker drift.
  - real-node profile requires signer profile summary/contracts markers:
    - `runtime_signer_profile_selector_env=KAMN_KOLME_LIVE_SIGNER_PROFILE`
    - `runtime_signer_profile=ops-primary`
    - `runtime_signer_previous_profile=ops-primary`
    - `runtime_signer_failover_active=false`
    - `runtime_signer_rotation_epoch=1`
    - `runtime_signer_previous_rotation_epoch=1`
    - `runtime_signer_key_source_contract_version=v1`
    - `runtime_signer_key_source=env-local`
    - `runtime_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX`
    - `runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF`
    - `runtime_signer_fallback_guard_contract_version=v2`
    - `runtime_signer_fallback_guard_mode=reject_if_present`
    - `runtime_signer_fallback_private_key_present=false`
    - `runtime_signer_raw_private_key_present=false`
    - `runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1`
    - `runtime_signer_attestation_bundle`
    - `runtime_signer_quorum_linkage_contract_version=v1`
    - `runtime_signer_quorum_required_approvals`
    - `runtime_signer_quorum_approved_signers_count`
    - `runtime_signer_quorum_profile_linked`
    - `runtime_signer_quorum_satisfied`
    - `runtime_signer_quorum_linked`
    - `contracts.runtime_signer_failover_requires_profile_change=true`
    - `contracts.runtime_signer_rotation_epoch_must_increase_on_failover=true`
    - `contracts.runtime_signer_quorum_linked_required=true`
    - `contracts.runtime_signer_quorum_threshold_required=true`
    - `contracts.runtime_signer_quorum_profile_membership_required=true`
    - `contracts.runtime_signer_fallback_guard_contract_version=v2`
    - `contracts.runtime_signer_fallback_guard_mode=reject_if_present`
    - `contracts.runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF`
    - `contracts.runtime_signer_fallback_private_key_allowed=false`
    - `contracts.runtime_signer_managed_external_raw_private_key_allowed=false`
  - real-node profile accepts secondary signer summary/contracts markers for failover drills:
    - `runtime_signer_profile=ops-secondary`
    - `runtime_signer_previous_profile=ops-secondary`
    - `runtime_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY`
    - `runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY; set KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY`
    - `contracts.runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY; set KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY`
    - `contracts.runtime_signer_failover_requires_profile_change=true`
    - `contracts.runtime_signer_rotation_epoch_must_increase_on_failover=true`
  - forced failover scenario matrix markers:
    - `runtime_signer_failover_active=true`
    - `runtime_signer_previous_profile=ops-primary`
    - `runtime_signer_rotation_epoch=2`
    - `runtime_signer_previous_rotation_epoch=1`
  - key-source/profile allowlist matrix contracts:
    - `ops-primary`: `env-local`, `managed-external`
    - `ops-secondary`: `env-local` only
  - real-node profile requires runtime command surfaces to include `KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1`; checker fails closed with `runtime_commit_real_signing_profile_marker_missing` when omitted.
  - real-node profile requires runtime command surfaces to include `KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary`; checker fails closed with `runtime_commit_signer_profile_marker_missing` when omitted.
  - real-node profile requires runtime command surfaces to include `KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary` when `runtime_signer_profile=ops-secondary`; checker fails closed with `runtime_commit_signer_profile_marker_missing` when omitted.
  - real-node profile requires managed-external runtime signer public-key command markers:
    - `ops-primary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX=<33-byte-compressed-secp256k1-hex>`
    - `ops-secondary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY=<33-byte-compressed-secp256k1-hex>`
    - lane command-surface checker missing marker fails closed with `runtime_commit_managed_external_signer_public_key_marker_missing`
    - `kamn-node` runtime managed-signer missing marker fails closed with `managed_signer_public_key_marker_missing`
    - invalid/empty/non-secp256k1 marker fails closed with `managed_signer_public_key_marker_invalid`
  - real-node profile command composition rejects in-memory fallback references at runner boundary:
    - `runtime-commit-command must not reference InMemoryKolmeRuntimeCommitClient when runtime-profile=real-node`
  - real-node profile command composition rejects fallback signer private-key command markers at runner boundary:
    - `runtime-commit-command must not include fallback signer private key marker KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK=... when runtime-profile=real-node`
  - real-node profile run-mode boundary rejects fallback signer secret env and prints remediation:
    - `fallback signer secret env must not be set: KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK (remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK)`
  - real-node profile managed-external boundary rejects raw signer secret env and prints remediation:
    - `managed-external signer raw private key env must not be set: KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX (remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF)`
  - real-node profile managed-external backend adapter command contract:
    - command env marker: `KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND`
    - managed-external signer mode requires command marker presence; missing marker fails closed with `managed_signer_backend_required_missing`
    - optional requirement override marker: `KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED=true|false`
    - invalid/empty requirement marker values fail closed with `managed_signer_backend_required_invalid`
    - runtime signer public-key env marker contracts:
      - `ops-primary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX`
      - `ops-secondary`: `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY`
      - missing marker fails closed with `managed_signer_public_key_marker_missing`
      - invalid/empty/non-secp256k1 marker fails closed with `managed_signer_public_key_marker_invalid`
    - optional timeout env marker: `KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS` (default `5`)
    - command input env markers: `KAMN_MANAGED_SIGNER_KEY_REFERENCE`, `KAMN_MANAGED_SIGNER_ACTOR_DID`, `KAMN_MANAGED_SIGNER_NONCE`, `KAMN_MANAGED_SIGNER_STATE_ROOT`, `KAMN_MANAGED_SIGNER_CANONICAL_MESSAGE`
    - command output markers: `signature_hex=<128-hex>`, `recovery_id=<0..3>`, and `signer_public_key_hex=<33-byte-compressed-secp256k1-hex>`
    - missing provenance marker fails closed with `managed_signer_backend_response_provenance_missing`
    - malformed provenance marker fails closed with `managed_signer_backend_response_provenance_malformed`
    - signer provenance mismatch fails closed with `managed_signer_backend_response_provenance_mismatch`
    - deterministic managed-external backend failure reason codes: `managed_signer_backend_timeout`, `managed_signer_backend_unavailable`, `managed_signer_backend_response_malformed`
  - real-node profile checker fails closed on in-memory provider references in command/policy surfaces:
    - `runtime_commit_in_memory_provider_reference_detected`
    - `runtime_commit_policy_check_in_memory_provider_reference_detected`
  - real-node signer-attestation checker fails closed on malformed attestations:
    - `runtime_signer_attestation_approved_signers_not_unique`
    - `runtime_signer_attestation_quorum_shortfall`
    - `runtime_signer_attestation_schema_invalid`
    - `runtime_signer_quorum_linkage_drift`
    - `runtime_signer_quorum_linkage_violation`
  - integration summary emits `ci_fast_gate_eligible=false` with `contracts.ci_fast_gate_scope=local-only` for explicit PR-fast-gate exclusion enforcement.
  - explicit runtime-commit submit-profile probe over `PUT /broadcast` with fail-closed reason codes.
  - signed runtime-commit envelope translation enforces `signer_key_id` presence and canonical message/signature binding before broadcast normalization.
  - finality verification uses `/notifications` first with bounded `/block/{height}` fallback; no runtime commit status endpoint dependency.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 210 seconds with per-stage budget caps.
  - local KAMN live runtime integration run-mode execution remains excluded from PR fast-gate workflow routing.
  - fallback signer key path remains fail-closed across runtime launch + wrapper/manifest entry points (`Regression: #2302`).
  - forced failover scenario matrix contracts remain fail-closed (`Regression: #2337`).
  - managed-external raw signer key path remains fail-closed across runtime launch + wrapper/manifest entry points (`Regression: #2324`).
  - runtime signer-attestation schema + quorum/uniqueness policy checks remain fail-closed across runtime launch + policy/contract lanes (`Regression: #2325`).
  - replay/tamper/stale-signer attestation regression matrix remains fail-closed (`Regression: #2327`).
  - real-node profile policy + contract lane docs parity markers remain fail-closed (`Regression: #2139`).

## Local Kolme Live Deployment Preflight Lane (Issue #2225)

- Local deployment preflight runner:
  - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Deployment preflight run mode:
  - `printf '%s\n' "custody-attestation=ops-primary:epoch-1" > /tmp/kolme-live-signer-custody.json`
  - `printf '%s\n' "signer-provenance=ops-primary:source-managed-external:epoch-1" > /tmp/kolme-live-signer-provenance.json`
  - `custody_sha="$(sha256sum /tmp/kolme-live-signer-custody.json | awk '{print $1}')"; cat > /tmp/kolme-live-signer-quorum.json <<JSON
{
  "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "required_approvals": 2,
  "received_approvals": 2,
  "approved_signers": ["ops-primary", "ops-secondary"],
  "signer_roles": {"ops-primary": "primary", "ops-secondary": "secondary"},
  "signer_rotation_epochs": {"ops-primary": 3, "ops-secondary": 2},
  "custody_evidence_sha256": "$custody_sha"
}
JSON`
  - `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=1111111111111111111111111111111111111111111111111111111111111111 bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode run --runtime-mode kolme-live --signer-profile ops-primary --required-approvals 2 --received-approvals 2 --custody-evidence-file /tmp/kolme-live-signer-custody.json --quorum-evidence-file /tmp/kolme-live-signer-quorum.json --signer-provenance-file /tmp/kolme-live-signer-provenance.json --signer-key-source managed-external --signer-key-source-contract-version v1 --signer-rotation-epoch 3 --signer-previous-rotation-epoch 1 --signer-rotation-freshness-max-delta 2 --max-seconds 12 --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_live_deployment_preflight_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_live_deployment_preflight_lane.json`
- Deployment preflight policy checker command:
  - `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code deployment_preflight_passed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Deployment preflight contract lane command:
  - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh --output-json /tmp/kolme-local-live-deployment-preflight-summary.json --policy-output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Summary/policy schemas:
  - summary: `kamn.kolme.local-live-deployment-preflight-summary.v1`
  - policy: `kamn.kolme.local-live-deployment-preflight-policy-report.v1`
- Deterministic checkpoints include:
  - `runtime_mode_contract`: runtime mode must match `kolme-live`.
  - `signer_profile_contract`: profile must be `ops-primary` or `ops-secondary`.
  - `signer_secret_contract`: selected signer secret env must be present and 64-char hex.
  - `fallback_private_key_contract`: fallback signer secret env must remain unset.
  - `signer_quorum_contract`: received approvals must satisfy required approvals threshold.
  - `quorum_evidence_contract`: quorum evidence bundle must satisfy schema, signer uniqueness, threshold, and custody digest match.
  - `custody_evidence_contract`: signer custody evidence file and sha256 marker must be present.
  - `signer_provenance_contract`: signer provenance evidence file and sha256 marker must be present.
  - `signer_rotation_freshness_contract`: signer rotation metadata must satisfy freshness threshold.
  - node runtime signer-provider guard (`KolmeLiveSignerSecretProvider`) rejects fallback env-key presence before key decode/signing.
  - summary fields include deterministic signer custody/provenance markers:
    - `signer_profile_selector_env=KAMN_KOLME_LIVE_SIGNER_PROFILE`
    - `signer_profile`
    - `signer_private_key_env`
    - `fallback_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`
    - `fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`
    - `fallback_signer_secret_present=false`
    - `signer_key_source_contract_version=v1`
    - `signer_key_source=managed-external`
    - `signer_provenance_file`
    - `signer_provenance_present`
    - `signer_provenance_sha256_valid`
    - `signer_rotation_epoch`
    - `signer_previous_rotation_epoch`
    - `signer_rotation_freshness_max_delta`
    - `signer_rotation_delta_epochs`
    - `signer_rotation_fresh`
    - `required_approvals=2`
    - `received_approvals`
    - `quorum_evidence_file`
    - `quorum_evidence_present`
    - `quorum_evidence_sha256_valid`
    - `quorum_evidence_schema_valid`
    - `quorum_evidence_approval_count`
    - `quorum_evidence_signers_unique`
    - `quorum_evidence_matches_threshold`
    - `quorum_evidence_custody_sha256_match`
    - `quorum_evidence_signer_roles_present`
    - `quorum_evidence_signer_roles_valid`
    - `quorum_evidence_rotation_metadata_present`
    - `quorum_evidence_rotation_metadata_valid`
    - `runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1`
    - `runtime_signer_attestation_bundle`
    - `runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1`
    - `runtime_signer_drift_telemetry`
    - `runtime_signer_drift_thresholds_schema_version=kamn.kolme.runtime-signer-drift-thresholds.v1`
    - `runtime_signer_drift_thresholds_bundle`
    - `custody_evidence_file`
    - `custody_evidence_present`
    - `custody_evidence_sha256_valid`
  - contracts include:
    - `contracts.ci_fast_gate_scope=ci-fast-gate`
    - `contracts.required_runtime_mode=kolme-live`
    - `contracts.required_secret_hex_length=64`
    - `contracts.fallback_private_key_path_allowed=false`
    - `contracts.fallback_signer_secret_rejected_profile_class=production`
    - `contracts.fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`
    - `contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract`
    - `contracts.approval_quorum_required=2`
    - `contracts.approval_quorum_minimum=2`
    - `contracts.approval_quorum_source=local-operator-attestations`
    - `contracts.quorum_evidence_required=true`
    - `contracts.quorum_evidence_sha256_required=true`
    - `contracts.quorum_evidence_schema_version=kamn.kolme.runtime-signer-attestation.v1`
    - `contracts.quorum_evidence_signer_uniqueness_required=true`
    - `contracts.quorum_evidence_custody_sha256_match_required=true`
    - `contracts.quorum_evidence_signer_roles_required=true`
    - `contracts.quorum_evidence_rotation_metadata_required=true`
    - `contracts.quorum_evidence_source=operator-attestation-bundle`
    - `contracts.runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1`
    - `contracts.runtime_signer_attestation_signer_uniqueness_required=true`
    - `contracts.runtime_signer_attestation_threshold_required=true`
    - `contracts.runtime_signer_attestation_profile_membership_required=true`
    - `contracts.runtime_signer_attestation_required_approvals=2`
    - `contracts.runtime_signer_drift_telemetry_required=true`
    - `contracts.runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1`
    - `contracts.runtime_signer_drift_telemetry_rotation_delta_match_required=true`
    - `contracts.runtime_signer_drift_telemetry_stale_flag_match_required=true`
    - `contracts.runtime_signer_drift_telemetry_quorum_flag_match_required=true`
    - `contracts.runtime_signer_drift_telemetry_approval_counts_match_required=true`
    - `contracts.runtime_signer_drift_thresholds_required=true`
    - `contracts.runtime_signer_drift_thresholds_schema_version=kamn.kolme.runtime-signer-drift-thresholds.v1`
    - `contracts.runtime_signer_drift_thresholds_rotation_warn_lte_fail_required=true`
    - `contracts.runtime_signer_drift_thresholds_quorum_warn_lte_fail_required=true`
    - `contracts.runtime_signer_drift_admission_matrix_required=true`
    - `contracts.runtime_signer_drift_admission_matrix_decision_values=GO,WARN,NO-GO`
    - `contracts.custody_evidence_required=true`
    - `contracts.custody_evidence_sha256_required=true`
    - `contracts.signer_provenance_required=true`
    - `contracts.signer_provenance_sha256_required=true`
    - `contracts.signer_key_source_contract_version=v1`
    - `contracts.signer_key_source=managed-external`
    - `contracts.required_signer_key_source_for_production=managed-external`
    - `contracts.signer_key_source_production_requirement_reason_code=signer_key_source_production_managed_external_required`
    - `contracts.signer_rotation_freshness_max_delta=2`
    - `contracts.signer_rotation_stale_rejected=true`
- Fail-closed reasons include:
  - `runtime_mode_mismatch`
  - `signer_profile_mismatch`
  - `signer_private_key_env_mismatch`
  - `fallback_signer_secret_present_violation`
  - `fallback_signer_secret_checkpoint_reason_mismatch`
  - `checkpoint_failed_signer_secret_contract`
  - `checkpoint_failed_signer_quorum_contract`
  - `checkpoint_failed_quorum_evidence_contract`
  - `checkpoint_failed_custody_evidence_contract`
  - `checkpoint_failed_signer_provenance_contract`
  - `checkpoint_failed_signer_rotation_freshness_contract`
  - `signer_quorum_shortfall`
  - `signer_quorum_minimum_not_met`
  - `quorum_evidence_missing`
  - `quorum_evidence_sha256_invalid`
  - `quorum_evidence_schema_invalid`
  - `quorum_evidence_signers_not_unique`
  - `quorum_evidence_signer_roles_missing`
  - `quorum_evidence_signer_roles_invalid`
  - `quorum_evidence_rotation_metadata_missing`
  - `quorum_evidence_rotation_metadata_invalid`
  - `quorum_evidence_approvals_mismatch`
  - `quorum_evidence_custody_sha256_mismatch`
  - `runtime_signer_attestation_approved_signers_not_unique`
  - `runtime_signer_attestation_quorum_shortfall`
  - `runtime_signer_attestation_schema_invalid`
  - `runtime_signer_drift_telemetry_missing`
  - `runtime_signer_drift_telemetry_schema_version_mismatch`
  - `runtime_signer_drift_telemetry_rotation_delta_invalid`
  - `runtime_signer_drift_admission_matrix_decision`
  - `runtime_signer_drift_admission_matrix_class`
  - `runtime_signer_drift_rotation_warning_threshold_reached`
  - `runtime_signer_drift_quorum_fail_threshold_exceeded`
  - `custody_evidence_missing`
  - `custody_evidence_sha256_invalid`
  - `signer_key_source_contract_version_mismatch`
  - `signer_key_source_invalid`
  - `signer_key_source_production_managed_external_required`
  - `signer_provenance_missing`
  - `signer_provenance_sha256_invalid`
  - `signer_rotation_epoch_stale`
- Cost policy:
  - lane is lightweight and `ci_fast_gate_eligible=true`.
  - no local-heavy opt-in is required for deployment preflight checks.
  - docs/command/policy parity for this lane remains fail-closed (`Regression: #2225`).
  - deployment preflight contract lane parity remains fail-closed (`Regression: #2226`).
  - fallback retirement docs parity remains fail-closed across README/CI/devnet runbooks (`Regression: #2337`).
  - signer provenance + rotation freshness marker parity remains fail-closed (`Regression: #2300`).
  - signer quorum evidence schema + custody digest parity remains fail-closed (`Regression: #2301`).
  - runtime/deployment shared signer-attestation schema + reason-code parity remains fail-closed (`Regression: #2326`).
  - replay/tamper/stale-signer attestation regression matrix remains fail-closed (`Regression: #2327`).

## Managed Signer Backend SLO Telemetry Lane (Issue #2436)

- Managed-signer backend SLO telemetry generator:
  - `bash scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh --output-file /tmp/managed-signer-backend-slo.json --window-start-utc 2026-02-13T00:00:00Z --window-end-utc 2026-02-13T00:15:00Z --backend-name kolme-managed-signer-primary --signer-profile ops-primary --signer-key-source managed-external --sample-count 100 --timeout-events 0 --unavailable-events 0 --error-events 1 --max-timeout-rate-bps 100 --max-unavailable-rate-bps 100 --max-error-rate-bps 200 --ci-fast-gate PASS`
- Managed-signer backend SLO contract lane:
  - `bash scripts/kolme/run_managed_signer_backend_slo_telemetry_contract_lane.sh --output-json /tmp/managed-signer-backend-slo-contract-report.json`
- Required schema/reason markers:
  - `kamn.kolme.managed-signer-backend-slo-telemetry.v1`
  - `signer_key_source=managed-external`
  - `contracts.required_signer_key_source=managed-external`
  - `managed_signer_backend_timeout_rate_threshold_exceeded`
  - `managed_signer_backend_unavailable_rate_threshold_exceeded`
  - `managed_signer_backend_error_rate_threshold_exceeded`
  - `managed_signer_backend_ci_fast_gate_failed`
- Cost policy:
  - lane remains lightweight and CI-fast-gate eligible.
  - artifacts are generated offline with deterministic thresholds and no external metrics backend dependency.

## Managed Signer Backend SLO Policy Lane (Issue #2437)

- Managed-signer backend SLO policy checker:
  - `python3 scripts/kolme/check_managed_signer_backend_slo_policy.py --telemetry-bundle /tmp/managed-signer-backend-slo.json --output-json /tmp/managed-signer-backend-slo-policy-report.json`
- Managed-signer backend SLO policy contract lane:
  - `bash scripts/kolme/run_managed_signer_backend_slo_policy_contract_lane.sh --output-json /tmp/managed-signer-backend-slo-policy-contract-report.json`
- Required schema/reason/remediation markers:
  - `kamn.kolme.managed-signer-backend-slo-policy-report.v1`
  - `kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1`
  - `managed_signer_backend_slo_within_threshold`
  - `managed_signer_backend_no_action_required`
  - `managed_signer_backend_timeout_rate_threshold_exceeded`
  - `managed_signer_backend_unavailable_rate_threshold_exceeded`
  - `managed_signer_backend_error_rate_threshold_exceeded`
  - `managed_signer_backend_ci_fast_gate_failed`
  - `managed_signer_backend_reduce_timeout_burst`
  - `managed_signer_backend_failover_endpoint`
  - `managed_signer_backend_enable_circuit_breaker`
  - `managed_signer_backend_replay_ci_fast_gate`
- Operator remediation guidance:
  - timeout bursts: gate promotion and reduce submit burst rate before rerun.
  - unavailable drift: fail over endpoint/profile before rerun.
  - error-rate drift: enable circuit breaker and hold promotion.
  - ci-fast-gate failure: rerun lane only after fast-gate blockers are resolved.
- Cost policy:
  - checker and contract lane remain local-fast and bounded.
  - no local-heavy selector or external metrics backend call is required.

## Managed Signer Startup Live Validation Contract Lane (Issue #3067)

- Managed-signer startup live validation contract lane:
  - `bash scripts/kolme/run_managed_signer_startup_live_validation_contract_lane.sh --output-json /tmp/managed-signer-startup-live-validation-contract-report.json`
- Required schema/decision markers:
  - `kamn.kolme.managed-signer-startup-live-validation-contract-report.v1`
  - `status=pass`
  - `final_decision=GO`
  - `managed_signer_profile_status=verified`
  - `managed_signer_missing_key_source_fail_closed_status=verified`
  - `managed_signer_invalid_profile_fail_closed_status=verified`
  - `managed_signer_stale_rotation_fail_closed_status=verified`
  - `managed_signer_reason_code_status=verified`
  - `execution_scope=local-scheduled`
- Fault-injection matrix and deterministic fail-closed reason codes:
  - missing managed-external key source:
    - checkpoint: `checkpoint_failed_signer_provenance_contract`
    - policy reason: `signer_key_source_production_managed_external_required`
  - invalid signer profile:
    - checkpoint: `checkpoint_failed_signer_profile_contract`
    - policy reason: `signer_profile_mismatch`
  - stale signer rotation metadata:
    - checkpoint: `checkpoint_failed_signer_rotation_freshness_contract`
    - policy reason: `signer_rotation_epoch_stale`
- Cost policy:
  - lane runtime is bounded via `--max-seconds`.
  - lane is local/scheduled by default (`ci_fast_gate_eligible=false`).
  - lane composes existing deployment preflight runner/policy checker and does not call external infrastructure.

## Staging Soak Telemetry Lane (Issue #2422)

- Fast lane command:
  - `bash scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- Deep/manual lane command:
  - `bash scripts/deploy/run_staging_rehearsal_deep_lane.sh`
- Direct bundle generation command (explicit telemetry thresholds):
  - `bash scripts/deploy/generate_staging_rehearsal_bundle.sh --output-file /tmp/staging-rehearsal-soak.json --release-candidate v1.1.0-rc.soak --deploy-status PASS --rollback-status PASS --rollback-target-hash state-hash-stable --post-rollback-hash state-hash-stable --recovery-time-seconds 420 --max-allowed-recovery-time-seconds 900 --evidence-complete true --ci-fast-gate PASS --runtime-submit-success-rate-bps 9950 --min-runtime-submit-success-rate-bps 9900 --runtime-finality-timeout-count 0 --max-runtime-finality-timeout-count 1 --signer-profile-drift-events 0 --max-signer-profile-drift-events 0`
- Policy checker command:
  - `bash scripts/deploy/check_staging_rehearsal_policy.sh --bundle-file /tmp/staging-rehearsal-soak.json`
- Telemetry threshold markers:
  - `runtime_submit_success_rate_bps` vs `min_runtime_submit_success_rate_bps`
  - `runtime_finality_timeout_count` vs `max_runtime_finality_timeout_count`
  - `signer_profile_drift_events` vs `max_signer_profile_drift_events`
- Deterministic threshold reason codes:
  - `runtime_submit_success_rate_below_threshold`
  - `runtime_finality_timeout_threshold_exceeded`
  - `signer_profile_drift_threshold_exceeded`
- Interpretation contract:
  - `final_decision=GO`: rollout soak telemetry remains within configured thresholds.
  - `final_decision=NO-GO`: at least one runtime telemetry threshold exceeded; capture `reason_codes` from policy output and block promotion until remediated.

## Live Provider Operator Runbook (Issue #2114)

### Prerequisites (Local)

- Local fork checkout exists at `/tmp/kolme_fork` (or a chosen path) with:
  - `origin` remote set to `https://github.com/njfio/kolme_fork.git`
  - symbolic `HEAD` resolving to `refs/heads/main`
- Local API endpoint for fork node is reachable at `http://127.0.0.1:3000` and responds to:
  - `GET /healthz`
  - `GET /fork-info?chain_version=v0.15.2`
- Local heavy opt-in is explicit for run mode:
  - `export KAMN_KOLME_LOCAL_HEAVY=1`
- Runtime signer custody contract envs are set for deployment preflight before integration execution:
  - `export KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary`
  - `export KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=<64-hex-private-key>`
  - `unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`
  - `printf '%s\n' "custody-attestation=ops-primary:epoch-1" > /tmp/kolme-live-signer-custody.json`
  - `printf '%s\n' "signer-provenance=ops-primary:source-managed-external:epoch-1" > /tmp/kolme-live-signer-provenance.json`
  - `custody_sha="$(sha256sum /tmp/kolme-live-signer-custody.json | awk '{print $1}')"; cat > /tmp/kolme-live-signer-quorum.json <<JSON
{
  "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "required_approvals": 2,
  "received_approvals": 2,
  "approved_signers": ["ops-primary", "ops-secondary"],
  "signer_roles": {"ops-primary": "primary", "ops-secondary": "secondary"},
  "signer_rotation_epochs": {"ops-primary": 3, "ops-secondary": 2},
  "custody_evidence_sha256": "$custody_sha"
}
JSON`

### Execution Flow

1. Run deployment preflight dry-run:
   - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
2. Validate deployment preflight policy:
   - `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
3. Run deployment preflight contract lane:
   - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh --output-json /tmp/kolme-local-live-deployment-preflight-summary.json --policy-output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
4. Dry-run integration plan:
   - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
5. Run integration lane (local-only):
   - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds 210 --bootstrap-max-seconds 90 --localhost-signed-max-seconds 45 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
6. Validate policy decision:
   - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`

Operator checkpoints:
- summary must include `ci_fast_gate_eligible=false`
- summary contracts must include `ci_fast_gate_scope=local-only`
- summary must include `runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider`
- deployment preflight summary must include `ci_fast_gate_eligible=true` with `contracts.ci_fast_gate_scope=ci-fast-gate`

### Rollback and Recovery Evidence

- If integration run fails, execute process lifecycle lane with explicit rollback/recovery evidence paths:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --integration-runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Confirm process lifecycle policy decision:
  - `python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
- Archive artifacts for release audit:
  - integration summary + policy
  - process lifecycle summary + policy
  - rollback/recovery evidence JSON files

### Troubleshooting

- `reason_code=local_opt_in_missing`:
  - set `KAMN_KOLME_LOCAL_HEAVY=1` and rerun.
- `reason_code=bootstrap_readiness_failed`:
  - re-run bootstrap lane directly and inspect checkout/probe markers.
- `reason_code=runtime_commit_policy_failed`:
  - inspect `/tmp/kolme-local-runtime-commit-live-policy.json` for provider marker or evidence mismatch.
- `reason_code=runtime_commit_in_memory_provider_reference_detected`:
  - treat as rollback condition; execute the process lifecycle lane with explicit `--rollback-evidence-file` and `--recovery-evidence-file` outputs before retrying runtime integration.
- `reason_code=runtime_commit_policy_check_in_memory_provider_reference_detected`:
  - treat as rollback condition; execute the process lifecycle lane with explicit `--rollback-evidence-file` and `--recovery-evidence-file` outputs before retrying runtime integration.
- `reason_code=checkpoint_failed_signer_secret_contract`:
  - verify signer profile and selected signer secret env are set with a valid 64-hex key.
- `reason_code=checkpoint_failed_signer_quorum_contract`:
  - verify `--received-approvals` is greater than or equal to `--required-approvals`.
- `reason_code=checkpoint_failed_quorum_evidence_contract`:
  - verify `--quorum-evidence-file` exists, `schema_version=kamn.kolme.runtime-signer-attestation.v1`, signer IDs are unique, approval counts match `--received-approvals`, and custody digest matches `--custody-evidence-file`.
- `reason_code=checkpoint_failed_custody_evidence_contract`:
  - verify `--custody-evidence-file` exists and its sha256 can be emitted in summary markers.
- `reason_code=checkpoint_failed_signer_provenance_contract`:
  - verify `--signer-provenance-file` exists and signer key-source markers remain `--signer-key-source managed-external --signer-key-source-contract-version v1`.
- `reason_code=checkpoint_failed_signer_rotation_freshness_contract`:
  - verify `--signer-rotation-epoch`, `--signer-previous-rotation-epoch`, and `--signer-rotation-freshness-max-delta` satisfy the freshness delta contract.
- `ci_fast_gate_eligibility_violation` or `ci_fast_gate_scope_mismatch` from policy checker:
  - verify summary still emits `ci_fast_gate_eligible=false` and `contracts.ci_fast_gate_scope=local-only`.

## Local Live-Node Validation Bundle Lane (Issue #2131)

- Deterministic local validation bundle plan:
  - `bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
- Wrapper routing remains manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_live_node_validation_bundle_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_live_node_validation_bundle_lane.json`
- Explicit local-only validation bundle execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_live_node_validation_bundle_policy.py --report-file /tmp/kolme-local-live-node-validation-bundle-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_live_node_validation_bundle_contract_lane.sh --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json --policy-output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
- Summary schema:
  - `kamn.kolme.local-live-node-validation-bundle-summary.v1`
- Policy schema:
  - `kamn.kolme.local-live-node-validation-bundle-policy-report.v1`
- Deterministic checkpoints include:
  - `integration_bundle`: local KAMN live runtime integration lane command composition with explicit `--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider`.
  - `integration_policy`: local KAMN live runtime integration policy decision checkpoint.
  - `process_lifecycle_bundle`: local fork process lifecycle lane command composition with rollback/recovery linkage.
  - `process_lifecycle_policy`: local fork process lifecycle policy decision checkpoint.
  - rollback/recovery lineage markers:
    - `rollback_evidence_file`
    - `recovery_evidence_file`
    - `rollback_evidence_file_missing`
    - `contracts.live_run_rehearsal_lineage_required=true`
    - `contracts.rollback_recovery_artifact_lineage_required=true`
    - `contracts.process_lifecycle_rollback_evidence_option=--rollback-evidence-file`
    - `contracts.process_lifecycle_recovery_evidence_option=--recovery-evidence-file`
- Decision contract:
  - policy returns GO only when schema, checkpoint markers, artifact lineage, and local-only boundary markers stay aligned with summary contracts.
  - policy fails closed to NO-GO on schema/evidence drift, missing checkpoints, provider marker drift, missing live run-lineage contracts, run-mode check status/reason-code drift (`run_mode_check_status_mismatch:*`, `run_mode_check_reason_code_mismatch:*`), or `ci_fast_gate_scope` mismatch.
- Cost policy:
  - bundle lane run mode remains local-only and requires explicit opt-in.
  - summary must emit `ci_fast_gate_eligible=false` and `contracts.ci_fast_gate_scope=local-only`.
  - run-mode execution remains excluded from PR fast-gate workflow routing.
  - contract lane remains dry-run-only and bounded for low-cost docs/command parity enforcement (`Regression: #2134`).

## On-Chain Lifecycle Aggregate Evidence Bundle Lane (Task #3249)

- Deterministic aggregate bundle plan:
  - `bash scripts/kolme/run_onchain_lifecycle_evidence_bundle_lane.sh --mode dry-run --did-report-file /tmp/kolme-did-lifecycle-live-validation-report.json --message-report-file /tmp/kolme-message-proof-live-validation-report.json --runtime-report-file /tmp/kolme-continuous-runtime-live-validation-report.json --output-json /tmp/kolme-onchain-lifecycle-evidence-bundle-summary.json`
- Wrapper routing remains manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_onchain_lifecycle_evidence_bundle_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_onchain_lifecycle_evidence_bundle_lane.json`
- Explicit local-only aggregate execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_onchain_lifecycle_evidence_bundle_lane.sh --mode run --output-json /tmp/kolme-onchain-lifecycle-evidence-bundle-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_onchain_lifecycle_evidence_policy.py check --report-file /tmp/kolme-onchain-lifecycle-evidence-bundle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-onchain-lifecycle-evidence-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_onchain_lifecycle_evidence_contract_lane.sh --output-json /tmp/kolme-onchain-lifecycle-evidence-bundle-summary.json --policy-output-json /tmp/kolme-onchain-lifecycle-evidence-policy.json`
- Summary schema:
  - `kamn.kolme.onchain-lifecycle-evidence-bundle.v1`
- Policy schema:
  - `kamn.kolme.onchain-lifecycle-evidence-policy-report.v1`
- Deterministic lineage fail-closed markers:
  - `aggregate_bundle_lineage_mismatch`
  - `finality_lineage_missing`
  - `recovery_lineage_missing`
  - `Regression: #3249`

## Milestone Review Aggregate Evidence Bundle (Issue #3247)

- Build linked release-governance artifacts:
  1. `bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
  2. `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
  3. `bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
  4. `python3 scripts/kolme/check_local_live_node_validation_bundle_policy.py --report-file /tmp/kolme-local-live-node-validation-bundle-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
  5. `bash scripts/runtime/run_go_no_go_gate_lane.sh --max-seconds 120 --output-json /tmp/go-no-go-gate-report.json`
- Generate deterministic milestone aggregate review bundle:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-milestone.json --release-candidate v1.0.0-rc.5 --schema-target-version 1.0.0 --runtime-image-digest sha256:abc123 --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --deployment-preflight-summary-file /tmp/kolme-local-live-deployment-preflight-summary.json --deployment-preflight-policy-file /tmp/kolme-local-live-deployment-preflight-policy.json --live-node-validation-summary-file /tmp/kolme-local-live-node-validation-bundle-summary.json --live-node-validation-policy-file /tmp/kolme-local-live-node-validation-bundle-policy.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json`
- Validate aggregate lineage policy:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-milestone.json`
- Aggregate bundle contracts:
  - `milestone_review_bundle.schema_version=kamn.release.milestone-review-bundle.v1`
  - `milestone_review_bundle.contracts.linked_artifact_lineage_required=true`
  - `milestone_review_bundle.contracts.live_bundle_runtime_provider_client_required=KolmeRuntimeCommitLiveProvider`
  - `milestone_review_bundle.contracts.go_no_go_gate_final_decision_required=GO`
  - `milestone_review_bundle.lineage_status=verified|fail-closed`
- Fail-closed reason markers:
  - `milestone_review_deployment_preflight_summary_missing`
  - `milestone_review_live_node_validation_summary_missing`
  - `milestone_review_go_no_go_gate_report_missing`
  - `milestone_review_live_node_validation_runtime_provider_mismatch`
  - `milestone_review_go_no_go_gate_final_decision_mismatch`
  - `milestone review bundle lineage mismatch`

## Staged Rehearsal Signoff Artifact Contract (Issue #3241)

- Rehearsal fast-lane contract command:
  - `bash scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- Rehearsal policy checker command:
  - `bash scripts/deploy/check_staging_rehearsal_policy.sh --bundle-file /tmp/staging-rehearsal-report.json`
- Signoff schema marker:
  - `kamn.release.staged-rehearsal-signoff.v1`
- Policy output marker:
  - `staged_rehearsal_signoff_status=verified|fail-closed`
- Fail-closed behavior:
  - deterministic signoff-schema or contract drift yields `staged rehearsal signoff artifact mismatch`.

## Localhost Two-Process Signed-Message Demo Contract (Issue #1612)

- Makefile demo command:
  - `make demo-localhost-transport`
- Direct localhost sender/listener demo command:
  - `bash scripts/sdk/run_localhost_signed_demo.sh --output-json /tmp/localhost-signed-demo-artifact.json`
- Demo artifact schema:
  - `kamn.sdk.localhost-signed.demo-receipt-artifact.v1`
- Localhost signed integration contract commands:
  - `bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json`
  - `bash scripts/sdk/check_localhost_signed_integration_evidence_policy.sh --report-file /tmp/localhost-signed-integration-contract-report.json`
- Integration contract schema:
  - `kamn.sdk.localhost-signed.integration-contract.v1`
- Localhost signed demo contract lane command:
  - `bash scripts/sdk/run_localhost_signed_demo_contract_lane.sh --output-json /tmp/localhost-signed-demo-contract-report.json`
- Shared report composer helper:
  - `scripts/sdk/localhost_signed_report_composer.py` (used by demo and integration contract wrappers to keep report schema/marker composition deterministic)
- Shared scenario runner helper:
  - `scripts/sdk/localhost_signed_scenario_runner.py` (used by integration contract wrapper to keep scenario execution deterministic with bounded timeout-race retries, signature-mismatch bounded retries, replay-nonce bounded retries, and admission bounded retries)
- Demo contract lane schema:
  - `kamn.sdk.localhost-signed.demo-contract.v1`
- Deterministic success markers:
  - `localhost signed message demo completed.`
  - `localhost signed integration contract lane tests passed.`
  - `localhost_signed_demo_status=pass`
  - `localhost_signed_integration_status=pass`
- Cost policy:
  - two-process localhost sender/listener demo remains bounded local smoke usage.
  - explicit local-heavy Kolme opt-in remains limited to heavy lanes and is not required for this demo path.
  - demo contract markers remain deterministic to keep onboarding checks local-fast.

## Unified Local Signed-to-Kolme Demo Contract Lane (Issue #1640)

- Unified local signed-to-Kolme demo runner:
  - `bash scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh --mode dry-run --output-json /tmp/kolme-local-signed-to-kolme-demo-summary.json`
- Explicit local-only unified demo execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh --mode run --max-seconds 420 --localhost-signed-demo-max-seconds 60 --localhost-signed-integration-max-seconds 120 --kolme-runtime-integration-max-seconds 300 --output-json /tmp/kolme-local-signed-to-kolme-demo-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_signed_to_kolme_demo_policy.py --report-file /tmp/kolme-local-signed-to-kolme-demo-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-signed-to-kolme-demo-policy.json`
- Summary schema:
  - `kamn.kolme.local-signed-to-kolme-demo-summary.v1`
- Deterministic checkpoints include:
  - `run_localhost_signed_demo_contract_lane.sh` run-mode verification before integration.
  - `run_localhost_signed_integration_contract_lane.sh` run-mode verification before Kolme runtime stage.
  - `run_local_kamn_live_runtime_integration_lane.sh` run-mode verification over localhost mock Kolme API before final GO decision.
  - explicit runtime-commit submit/finality marker contracts: `runtime_commit_submit_evidence_marker_present`, `runtime_commit_finality_evidence_marker_present`.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane enforces bounded per-stage and total runtime budgets.
  - run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Kolme Fork Process Lifecycle Integration Lane (Issue #1494)

- Local fork process lifecycle runner:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Explicit local-only process lifecycle execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --max-seconds 300 --startup-max-seconds 45 --integration-max-seconds 240 --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_process_lifecycle_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_lane.json`
- Optional nested integration runtime finality/policy pass-through:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --integration-runtime-commit-finality-command "printf 'finality=final\n'" --integration-runtime-commit-finality-max-seconds 15 --integration-runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --integration-runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-process-lifecycle-summary.v1`
- Deterministic checkpoints include:
  - process command orchestration: start -> readiness probe -> nested `run_local_kamn_live_runtime_integration_lane.sh` -> teardown.
  - optional integration runtime finality pass-through (`--integration-runtime-commit-finality-command`, `--integration-runtime-commit-finality-max-seconds`, `--integration-runtime-commit-finality-output-file`) is forwarded to nested integration lane runtime finality options.
  - integration runtime policy pass-through (`--integration-runtime-commit-live-policy-report`) is forwarded to nested integration lane runtime policy artifact output.
  - rollback/recovery evidence linkage artifacts (`--rollback-evidence-file`, `--recovery-evidence-file`) are deterministic summary outputs for release-hardening decision traces.
  - readiness contract over `GET /healthz` and `GET /fork-info?chain_version=<version>`.
  - fail-closed reason codes for local opt-in, serve-command, bootstrap, readiness, integration, teardown, and budget drift.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 300 seconds with per-stage integration budget caps.
  - local fork process lifecycle integration run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Fork Profile Preflight Lane (Issues #1648, #1696)

- Local fork profile preflight runner:
  - `bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-profile-preflight-summary.json`
- Explicit local-only preflight execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode run --checkout-path /tmp/kolme_fork --max-seconds 45 --output-json /tmp/kolme-local-fork-profile-preflight-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_profile_preflight_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_profile_preflight_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file /tmp/kolme-local-fork-profile-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-profile-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-profile-preflight-summary.v1`
  - policy schema: `kamn.kolme.local-fork-profile-preflight-policy-report.v1`
- Deterministic checkpoints include:
  - default profile contract requires `cd /tmp/kolme_fork && cargo run --bin example-six-sigma -- serve api-server`.
  - fail-closed policy gate for checkout/profile drift unless explicit local harness override is supplied.
  - bounded probe command execution with deterministic reason-code outcomes.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - default budget is bounded to 45 seconds.
  - preflight run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Fork Self-Test Lane (Issues #1652, #1701)

- Local fork self-test runner:
  - `bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-self-test-summary.json`
- Explicit local-only self-test execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --matrix-max-seconds 60 --matrix-cargo-profile portable --output-json /tmp/kolme-local-fork-self-test-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_self_test_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_self_test_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file /tmp/kolme-local-fork-self-test-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-self-test-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_self_test_contract_lane.sh --output-json /tmp/kolme-local-fork-self-test-summary.json --policy-output-json /tmp/kolme-local-fork-self-test-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-self-test-summary.v1`
  - policy schema: `kamn.kolme.local-fork-self-test-policy-report.v1`
- Deterministic checkpoints include:
  - `run_local_kolme_fork_rust_test_matrix_lane.sh` run-mode verification with bounded matrix budget, optional command overrides, and configurable cargo profile (`strict|portable`).
  - `check_local_kolme_fork_rust_test_matrix_policy.py` GO decision verification with required reason-code contract.
  - fail-closed reason codes for local opt-in, nested matrix failure, nested policy failure, and total-budget drift.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - nested matrix budget and total lane budget remain bounded and local-only.
  - self-test run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Fork Portability Preflight Lane (Issue #1706)

- Local fork portability preflight runner:
  - `bash scripts/kolme/run_local_kolme_fork_portability_preflight_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-portability-preflight-summary.json`
- Explicit local-only portability preflight execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_portability_preflight_lane.sh --mode run --checkout-path /tmp/kolme_fork --max-seconds 300 --output-json /tmp/kolme-local-fork-portability-preflight-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_portability_preflight_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_portability_preflight_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_portability_preflight_policy.py --report-file /tmp/kolme-local-fork-portability-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-portability-preflight-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_portability_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-portability-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-portability-preflight-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-portability-preflight-summary.v1`
  - policy schema: `kamn.kolme.local-fork-portability-preflight-policy-report.v1`
- Deterministic checkpoints include:
  - local-only opt-in guard before active probes.
  - `mold` linker probe when checkout toolchain config requires `-fuse-ld=mold`.
  - portable `kolme` compile probe using `RUSTFLAGS=''`.
  - `libudev` probe prior to integration-test compile probe.
  - `integration-tests` compile probe (`six-sigma`) to surface host dependency drift.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - compile probes remain local-only and bounded by explicit runtime budgets.
  - portability preflight run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Fork Checkout Bootstrap Lane (Issue #1663)

- Local fork checkout bootstrap runner:
  - `bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --expected-commit 0000000000000000000000000000000000000000 --fork-pin-manifest-file fixtures/kolme_compatibility/kolme_fork_pin_manifest.json --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json`
- Explicit local-only checkout bootstrap execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --expected-commit <40-hex-pinned-sha> --fork-pin-manifest-file /tmp/kolme-fork-pin-manifest.json --max-seconds 120 --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_checkout_bootstrap_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_lane.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py --report-file /tmp/kolme-local-fork-checkout-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json --policy-output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-checkout-bootstrap-summary.v1`
- Deterministic checkpoints include:
  - checkout preparation (clone/update) against pinned `fork-remote-url`.
  - nested `run_local_fork_sync_metadata_lane.sh` run-mode validation for remote/ref/commit provenance.
  - commit pin contract fields:
    - `expected_commit`
    - `commit_pin_enforced=true`
    - `fork_pin_manifest_schema_version=kamn.kolme.fork-pin-manifest.v1`
  - deterministic diagnostics capture for `git --version`, `cargo --version`, and `rustc --version`.
  - fail-closed reason codes for missing local opt-in, checkout bootstrap failure, metadata drift, diagnostics failure, and runtime budget overrun.
  - pinned commit drift reason marker:
    - `head_commit_mismatch`
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget remains bounded to 120 seconds.
  - run-mode execution remains local-only and excluded from PR fast-gate workflow routing.
  - pinned fork remote/ref/commit manifest and drift checker contracts remain fail-closed (`Regression: #2328`).

## Real Fork Local Process Wrapper Contract Lane (Issue #1644)

- Real-fork local wrapper runner:
  - `bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-real-process-summary.json`
- Explicit local-only real-fork wrapper execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 360 --bootstrap-max-seconds 120 --preflight-max-seconds 45 --self-test-max-seconds 120 --self-test-matrix-max-seconds 60 --lifecycle-max-seconds 300 --lifecycle-startup-max-seconds 45 --lifecycle-integration-max-seconds 240 --lifecycle-bootstrap-max-seconds 90 --lifecycle-conformance-max-seconds 180 --lifecycle-runtime-commit-max-seconds 30 --output-json /tmp/kolme-local-fork-real-process-summary.json`
- Optional lifecycle runtime finality pass-through:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --lifecycle-mode run --lifecycle-runtime-commit-finality-command "printf 'finality=final\n'" --lifecycle-runtime-commit-finality-max-seconds 15 --lifecycle-runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --lifecycle-rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --lifecycle-recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-real-process-summary.json`
- Default serve profile contract:
  - `cd <checkout-path> && cargo run --bin example-six-sigma -- serve api-server`
- Checkout bootstrap prerequisite commands:
  - `bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json`
  - `python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py --report-file /tmp/kolme-local-fork-checkout-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code fork_checkout_bootstrap_passed --output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json`
- Profile preflight prerequisite commands:
  - `bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode run --checkout-path /tmp/kolme_fork --max-seconds 45 --output-json /tmp/kolme-local-fork-profile-preflight-summary.json`
  - `python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file /tmp/kolme-local-fork-profile-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code profile_preflight_passed --output-json /tmp/kolme-local-fork-profile-preflight-policy.json`
- Local self-test prerequisite commands:
  - `bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --matrix-max-seconds 60 --matrix-cargo-profile portable --output-json /tmp/kolme-local-fork-self-test-summary.json`
  - `python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file /tmp/kolme-local-fork-self-test-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code fork_self_test_passed --output-json /tmp/kolme-local-fork-self-test-policy.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_real_process_policy.py --report-file /tmp/kolme-local-fork-real-process-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-real-process-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-real-process-summary.v1`
- Deterministic checkpoints include:
  - real-fork command profile validation for `example-six-sigma serve api-server`.
  - `run_local_kolme_fork_checkout_bootstrap_lane.sh` and policy verification execute before preflight/self-test/lifecycle orchestration.
  - `run_local_kolme_fork_profile_preflight_lane.sh` and policy verification execute before self-test/lifecycle orchestration.
  - `run_local_kolme_fork_self_test_lane.sh` and policy verification execute before lifecycle orchestration.
  - `run_local_kolme_fork_process_lifecycle_lane.sh` run-mode composition with bounded budgets.
  - lifecycle mode selector (`--lifecycle-mode dry-run|run`) controls whether nested process lifecycle lane executes planning-only flow or local run orchestration.
  - optional lifecycle runtime finality pass-through (`--lifecycle-runtime-commit-finality-command`, `--lifecycle-runtime-commit-finality-max-seconds`, `--lifecycle-runtime-commit-finality-output-file`) is forwarded into nested process lifecycle integration finality options.
  - lifecycle rollback/recovery pass-through (`--lifecycle-rollback-evidence-file`, `--lifecycle-recovery-evidence-file`) is forwarded into nested process lifecycle rollback/recovery evidence linkage options.
  - `check_local_kolme_fork_process_lifecycle_policy.py` GO decision verification.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - wrapper run mode enforces bounded preflight/self-test/lifecycle and total runtime budgets.
  - wrapper run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Runtime Commit Live Proof Lane (Issue #1450)

- Local runtime-commit live lane runner:
  - `bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode dry-run --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Explicit opt-in live execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --base-url http://127.0.0.1:3000 --provider-hint kolme-fork-local --max-seconds 90 --preflight-max-seconds 10 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Optional post-submit finality follow-up execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --skip-preflight --live-command "printf 'status=submitted\\nreplay_guard=verified\\n'" --finality-command "printf 'finality=final\\n'" --finality-max-seconds 15 --finality-retry-max-attempts 2 --finality-retry-backoff-seconds 0 --max-seconds 90 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt --finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt`
- Evidence policy checker command:
  - `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-runtime-commit-live-policy.json`
  - `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --output-json /tmp/kolme-local-runtime-commit-live-policy.json`
- Finality evidence contract lane command:
  - `bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json`
- Bounded localhost live-provider integration proof lane:
  - `bash scripts/kolme/run_local_live_provider_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-live-provider-runtime-integration-contract-report.json`
- Default live-provider smoke command executed by run mode:
  - `KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --exact integration_kolme_fork_live_node_submit_reaches_endpoint`
- Summary schema:
  - `kamn.kolme.local-runtime-commit-live-summary.v1`
  - policy schema: `kamn.kolme.local-runtime-commit-live-policy-report.v1`
- Deterministic checkpoints include:
  - live-provider pipeline ownership is canonical in `crates/kamn-kolme/src/live_provider_pipeline.rs`; `crates/kamn-core/src/kolme_runtime_commit/live_provider.rs` remains a compatibility facade.
  - wrapper routing remains manifest-backed via `scripts/kolme/run_lane_dispatch.sh` resolving `scripts/framework/manifests/kolme_local_runtime_commit_live_lane.json`.
  - bounded preflight probe against `<base-url>/healthz` before live submit execution (unless `--skip-preflight` is explicitly set)
  - explicit local-only opt-in marker (`KAMN_KOLME_LOCAL_HEAVY=1`)
  - bounded live command timeout via `--max-seconds`
  - optional finality command timeout bound via `--finality-max-seconds`
  - bounded finality retry controls via `--finality-retry-max-attempts` and `--finality-retry-backoff-seconds`
  - submit/finality/replay evidence marker fields (`submit_evidence_marker_present`, `finality_evidence_marker_present`, `replay_evidence_marker_present`) remain fail-closed in policy checks
  - replay marker contract field (`replay_evidence_contract_version`) remains fail-closed in policy checks
  - request/finality linkage markers (`request_payload_evidence_marker_present`, `request_payload_evidence_artifact_path`, `submit_evidence_artifact_path`, `finality_evidence_artifact_path`, `request_finality_evidence_contract_version`, `request_finality_evidence_linked`) remain fail-closed in policy checks
  - finality retry evidence markers (`finality_retry_contract_version`, `finality_retry_max_attempts`, `finality_retry_backoff_seconds`, `finality_retry_attempts_used`, `finality_retry_exhausted`, `finality_retry_failure_class`) remain fail-closed in policy checks
  - live-provider marker contracts (`provider_contract_enforcement_mode`, `provider_live_contract_marker`, `provider_live_contract_marker_present`, `provider_in_memory_reference_detected`) remain fail-closed in policy checks
  - real-signing marker contracts (`provider_signer_adapter_contract=KolmeForkSecp256k1SignerAdapter`, `provider_signing_curve_contract=secp256k1`, `provider_signing_profile_contract_version=v1`) remain fail-closed in policy checks
  - provider mismatch remains fail-closed with deterministic reason `provider_client_contract_mismatch`.
  - signer-adapter drift remains fail-closed with deterministic reason `provider_signer_adapter_contract_mismatch`.
  - bounded localhost integration proof lane validates deterministic NO-GO on node-unavailable preflight reasons `live_preflight_failed` or `live_preflight_timeout`.
  - native payload evidence marker fields (`native_payload_pubkey_marker_present`, `native_payload_nonce_marker_present`, `native_payload_messages_marker_present`) remain fail-closed in strict real-node policy checks
  - summary includes `live_command_synthetic`, `finality_command_synthetic`, and `synthetic_evidence_classification_version=v1` for deterministic synthetic-command detection.
  - `--require-non-synthetic-run-evidence` enforces NO-GO on synthetic run-mode command paths (`synthetic_live_command_detected`, `synthetic_finality_command_detected`).
  - `--require-native-payload-evidence` enforces NO-GO when run-mode native payload markers are absent (`native_payload_pubkey_marker_missing`, `native_payload_nonce_marker_missing`, `native_payload_messages_marker_missing`).
  - request/finality linkage drift fails closed with deterministic reasons (`request_payload_evidence_marker_missing`, `replay_evidence_marker_missing`, `finality_evidence_artifact_path_missing`, `request_finality_evidence_linkage_missing`).
  - provider drift fails closed when in-memory provider usage is detected in summary marker surfaces (`provider_in_memory_reference_detected`).
  - finality retry exhaustion reasons are deterministic (`live_finality_retry_exhausted_timeout`, `live_finality_retry_exhausted_failed`) and drift fails closed with checker reasons (`finality_retry_failure_class_mismatch_for_timeout_reason`, `finality_retry_attempts_used_mismatch_for_timeout_reason`).
  - machine-readable pass/fail reason codes for missing opt-in, preflight failure/timeout, command failure, and command timeout
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - live command timeout/exceeded budget is reported as `live_runtime_commit_command_timeout`.
  - preflight failures are reported as `live_preflight_failed` or `live_preflight_timeout`.

## Local Native API Parity Live Proof Lane (Issue #1465)

- Native API parity live-proof lane runner:
  - `bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode dry-run --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json`
- Wrapper routing remains manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_native_api_parity_live_proof_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_native_api_parity_live_proof_lane.json`
- Explicit opt-in live proof execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode run --nonce-command "curl --silent --show-error --fail http://127.0.0.1:3000/get-next-nonce?pubkey=test-key" --broadcast-command "curl --silent --show-error --fail --request PUT --data '{\"message\":\"native-parity\",\"signature\":\"sig\",\"recovery_id\":1}' http://127.0.0.1:3000/broadcast" --finality-command "curl --silent --show-error --fail http://127.0.0.1:3000/block/1" --max-seconds 180 --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_native_api_parity_live_proof_policy.py --report-file /tmp/kolme-local-native-api-parity-live-proof-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-native-api-parity-live-proof-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_native_api_parity_live_proof_contract_lane.sh --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json --policy-output-json /tmp/kolme-local-native-api-parity-live-proof-policy.json`
- Summary schema:
  - `kamn.kolme.local-native-api-parity-live-proof-summary.v1`
- Deterministic checkpoints include:
  - explicit local-only opt-in marker (`KAMN_KOLME_LOCAL_HEAVY=1`)
  - bounded native parity proof budget via `--max-seconds`
  - deterministic pass/fail reason codes for missing opt-in, command missing, timeout, and command failures
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 180 seconds.

## Fast-Gate Native API Parity Contract Lane (Issues #1466, #1468)

- Fast-gate parity lane runner:
  - `bash scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh --output-json /tmp/kolme-fast-gate-native-api-parity-summary.json`
- Fast-gate parity policy checker:
  - `python3 scripts/kolme/check_fast_gate_native_api_parity_policy.py --report-file /tmp/kolme-fast-gate-native-api-parity-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-fast-gate-native-api-parity-policy.json`
- Summary schema:
  - `kamn.kolme.fast-gate-native-api-parity-summary.v1`
- Deterministic checkpoints include:
  - bounded composition of nonce/broadcast parity, notifications consumer, and block fallback contract lanes
  - fail-closed reason codes for command timeout/failure and budget overrun
- Cost policy:
  - lane default budget is bounded to 120 seconds via `KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS`.
  - local-heavy run-mode parity commands remain excluded from PR fast-gate workflow routing.

## Secp256k1 Signature Parity Fixture Lane (Issue #2345)

- Fixture-backed matrix runner:
  - `python3 scripts/kolme/run_signature_parity_matrix.py --fixture fixtures/kolme_commit/signature_parity_vectors.json --output-json /tmp/kolme-signature-parity-matrix-report.json`
- Policy checker:
  - `python3 scripts/kolme/check_signature_parity_policy.py --report-file /tmp/kolme-signature-parity-matrix-report.json --expected-final-decision GO --ci-fast-gate PASS --require-vector-id kolme_fork_primary_alpha --require-vector-id kolme_fork_secondary_beta --require-vector-id kolme_fork_primary_alpha_bad_signature --require-vector-id kolme_fork_secondary_beta_bad_recovery --require-vector-id kolme_fork_primary_alpha_bad_pubkey --output-json /tmp/kolme-signature-parity-policy-report.json`
- Contract lane wrapper:
  - `bash scripts/kolme/run_signature_parity_contract_lane.sh --output-json /tmp/kolme-signature-parity-matrix-report.json --policy-output-json /tmp/kolme-signature-parity-policy-report.json`
- Deterministic negative-vector checkpoints:
  - `kolme_fork_primary_alpha_bad_signature` must emit `parity_signature_mismatch`.
  - `kolme_fork_secondary_beta_bad_recovery` must emit `parity_recovery_id_mismatch`.
  - `kolme_fork_primary_alpha_bad_pubkey` must emit `parity_pubkey_mismatch`.
- Policy fail-closed checkpoints:
  - NO-GO cases must include deterministic `reason_codes`.
  - unknown/unrecognized case-level reason codes are rejected.

## Deterministic Local Bootstrap Health Checks (Issues #1417, #1691)

- Bootstrap health-check runner:
  - `bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode dry-run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Explicit opt-in bootstrap execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Policy checker contract:
  - `python3 scripts/kolme/check_local_bootstrap_health_policy.py --report-file /tmp/kolme-local-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-bootstrap-policy.json`
- Bounded contract lane (dry-run + policy):
  - `bash scripts/kolme/run_local_bootstrap_health_checks_contract_lane.sh --output-json /tmp/kolme-local-bootstrap-summary.json --policy-output-json /tmp/kolme-local-bootstrap-policy.json`
- Summary schema:
  - `kamn.kolme.local-bootstrap-summary.v1`
  - policy schema: `kamn.kolme.local-bootstrap-policy-report.v1`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_bootstrap_health_checks.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_bootstrap_health_checks_lane.json`
- Deterministic readiness checks include:
  - `validate_version_compatibility.py`
  - `generate_fork_compatibility_evidence.py`
  - `check_fork_compatibility_policy.py`
  - `run_triadic_devnet_smoke.sh`
  - `validate_triadic_devnet_smoke.py`
  - deterministic dry-run reason code: `dry_run_no_commands_executed`
  - deterministic run-mode success reason code: `local_bootstrap_health_checks_passed`
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.

## Local-Only Heavy End-to-End Lane (Issue #1418)

- Local-only E2E lane runner:
  - `bash scripts/kolme/run_local_e2e_integration_lane.sh --mode dry-run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Explicit opt-in E2E execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_e2e_integration_lane.sh --mode run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Policy checker contract:
  - `python3 scripts/kolme/check_local_e2e_integration_policy.py --report-file /tmp/kolme-local-e2e-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-e2e-integration-policy.json`
- Bounded contract lane (dry-run + policy):
  - `bash scripts/kolme/run_local_e2e_integration_contract_lane.sh --output-json /tmp/kolme-local-e2e-integration-summary.json --policy-output-json /tmp/kolme-local-e2e-integration-policy.json`
- Summary schema:
  - `kamn.kolme.local-e2e-integration-summary.v1`
  - policy schema: `kamn.kolme.local-e2e-integration-policy-report.v1`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_e2e_integration_lane.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_e2e_integration_lane.json`
- Deterministic checkpoints include:
  - `run_local_bootstrap_health_checks.sh`
  - `run_local_kolme_fork_checkout_bootstrap_contract_lane.sh`
  - `run_runtime_commit_adapter_contract_lane.sh`
  - `run_live_transport_parity_contract_lane.sh --languages python,typescript`
  - `run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
  - `run_local_kolme_live_api_conformance_contract_lane.sh`
  - shared JSON summary generation via `scripts/framework/generate_local_lane_summary.py`
- Cost policy:
  - lane enforces explicit local-only opt-in and a deterministic runtime budget ceiling.
  - shared opt-in enforcement helper: `scripts/framework/assert_local_heavy_opt_in.sh`.

## Local-Only Heavy Kolme Validation Matrix (Issue #1405)

- Local-only matrix runner:
  - `bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Explicit opt-in execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Policy checker contract:
  - `python3 scripts/kolme/check_local_heavy_validation_matrix_policy.py --report-file /tmp/kolme-local-heavy-validation-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-heavy-validation-policy.json`
- Run-mode policy checker contract (after explicit opt-in execution):
  - `python3 scripts/kolme/check_local_heavy_validation_matrix_policy.py --report-file /tmp/kolme-local-heavy-validation-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code local_heavy_validation_passed --output-json /tmp/kolme-local-heavy-validation-policy.json`
- Bounded contract lane (dry-run + policy):
  - `bash scripts/kolme/run_local_heavy_validation_matrix_contract_lane.sh --output-json /tmp/kolme-local-heavy-validation-summary.json --policy-output-json /tmp/kolme-local-heavy-validation-policy.json`
- Summary schema:
  - `kamn.kolme.local-heavy-validation-summary.v1`
  - policy schema: `kamn.kolme.local-heavy-validation-policy-report.v1`
- Wrapper routing stays manifest-backed:
  - `scripts/kolme/run_lane_dispatch.sh --lane-wrapper run_local_heavy_validation_matrix.sh --resolve-manifest-path`
  - `scripts/framework/manifests/kolme_local_heavy_validation_matrix_lane.json`
- Heavy command set includes:
  - `scripts/kolme/run_local_bootstrap_health_checks.sh`
  - `scripts/kolme/run_version_compatibility_replay_deep_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh`
  - shared JSON summary generation via `scripts/framework/generate_local_lane_summary.py`
- Cost policy:
  - matrix execution remains local-only and is excluded from PR fast-gate workflow routing.
  - shared opt-in enforcement helper: `scripts/framework/assert_local_heavy_opt_in.sh`.

## Runtime Local Three-Node Convergence (Issue #3417)

- Bounded dry-run contract lane:
  - `bash scripts/runtime/validate_local_full_runtime_live_contract_lane.sh --mode dry-run --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy-report.json`
- Explicit local-heavy run mode:
  - `KAMN_LOCAL_FULL_RUNTIME_LIVE_OPT_IN=1 bash scripts/runtime/validate_local_full_runtime_live.sh --mode run --ci-fast-gate FAIL --output-json /tmp/local-full-runtime-live-summary.json`
- Required convergence evidence markers:
  - `three_node_role_set_status=verified`
  - `transport_propagation_status=verified`
  - `canonical_convergence_status=verified`
  - `runtime_transport_mode=libp2p_transport_fed`
- Deterministic fail-closed policy reason markers:
  - `local_full_runtime_policy_runtime_transport_mode_mismatch`
  - `local_full_runtime_policy_three_node_role_set_status_mismatch`
  - `local_full_runtime_policy_canonical_convergence_status_mismatch`

## Runtime Block Reconciliation Partition/Rejoin (Issue #3418)

- Bounded dry-run contract lane:
  - `bash scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh --mode dry-run --ci-fast-gate PASS --output-json /tmp/block-reconciliation-partition-rejoin-contract-lane-report.json --policy-output-json /tmp/block-reconciliation-partition-rejoin-policy-report.json`
- Explicit local-heavy run mode:
  - `KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_OPT_IN=1 bash scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh --mode run --ci-fast-gate FAIL --output-json /tmp/block-reconciliation-partition-rejoin-live-summary.json`
- Required reconciliation evidence markers:
  - `runtime_transport_mode=libp2p_transport_fed`
  - `transport_state_transition_status=verified`
  - `head_alignment_status=verified`
  - `quorum_restore_status=verified`
  - `replay_stabilization_status=verified`
  - `publish_drop_recovery_status=verified`
  - `peer_churn_recovery_status=verified`
  - `reconciliation_reason_taxonomy_status=verified`
  - `reconciliation_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1`
  - `reconciliation_reason_codes=none|reconciliation_partition_transition_failed|reconciliation_rejoin_transition_failed|reconciliation_publish_drop_recovery_failed|reconciliation_peer_churn_recovery_failed|reconciliation_split_head_unresolved|reconciliation_replay_instability|...`
- Deterministic fail-closed policy reason markers:
  - `block_reconciliation_partition_rejoin_policy_transport_mode_mismatch`
  - `block_reconciliation_partition_rejoin_policy_head_alignment_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_quorum_restore_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_replay_stabilization_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_publish_drop_recovery_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_peer_churn_recovery_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_version_mismatch`
  - `block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid`

## Failover + Sync Drill Lane Policy (Issues #787, #788)

- Selector policy:
  - `bash scripts/runtime/select_failover_sync_drill_lane.sh --event-name pull_request`
  - `pull_request` routes to `preflight`; `schedule` and `workflow_dispatch` route to `deep`.
- PR-fast preflight lane:
  - `bash scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh --output-json /tmp/failover-sync-preflight-report.json`
  - preflight lane enforces a bounded runtime budget (default 15 seconds).
- Scheduled deep lane:
  - `KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled bash scripts/runtime/run_failover_sync_drill_deep_lane.sh --output-json /tmp/failover-sync-deep-report.json`
  - deep lane fails closed when invoked without scheduled cadence marker.
- CI-oriented suite entrypoint:
  - `bash scripts/runtime/run_failover_sync_drill_suite.sh --event-name schedule --output-json /tmp/failover-sync-suite-report.json`
  - suite report schema: `kamn.runtime.failover-sync-drill-suite-report.v1`.

## Regression Guard

- Marker drift remains fail-closed via fixture-backed validation (`Regression: #785`).
- runtime commit adapter replay/finality reason-code drift fails closed (`Regression: #980`).
- deterministic bootstrap run mode fails closed without explicit local-only opt-in (`Regression: #1417`).
- local-only heavy E2E lane run mode fails closed without explicit local-only opt-in (`Regression: #1418`).
- local-only heavy E2E lane summary policy and contract-lane decision/checkpoint drift remain fail-closed (`Regression: #1682`).
- shared local-heavy opt-in helper wiring remains fail-closed across bootstrap/E2E/matrix lanes (`Regression: #1585`).
- local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`).
- local-only heavy validation matrix summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1687`).
- local bootstrap health summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1692`).
- lane migration matrix schema/order/required-lane drift remains fail-closed (`Regression: #1721`).
- tranche-1 manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1722`).
- tranche-1 wrapper/direct manifest execution parity drift remains fail-closed (`Regression: #2118`).
- runtime+nonce manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1763`).
- version+matrix manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1765`).
- profile+self-test+portability manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1767`).
- runtime+triadic+bootstrap+e2e manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1769`).
- bootstrap+conformance+runtime+process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1771`).
- parity+demo+real-process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1773`).
- local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`).
- local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`).
- local fork Rust test matrix lane fails closed on missing local opt-in, metadata sync drift, and per-command timeout/failure paths (`Regression: #1537`).
- local fork Rust test matrix portable cargo profile (`--cargo-profile portable`) remains fail-closed and linker-portable via `RUSTFLAGS=''` cargo override (`Regression: #1659`).
- local fork Rust test matrix policy and contract-lane checks fail closed on schema/decision/reason-code drift (`Regression: #1541`).
- local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`).
- local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`).
- local live API conformance harness fails closed for probe/native parity prerequisite failures, runtime budget overruns, and endpoint contract drift (`Regression: #1483`).
- local fork bootstrap/readiness lane fails closed for sync/probe prerequisite failures, runtime budget overruns, and missing local opt-in (`Regression: #1488`).
- local KAMN live runtime integration lane fails closed for bootstrap/localhost-signed/conformance/runtime-commit prerequisite drift, runtime budget overruns, and missing local opt-in (`Regression: #1489`).
- local KAMN live runtime integration lane propagates runtime finality pass-through options and artifacts to nested runtime live lane command composition (`Regression: #1971`).
- local KAMN live runtime integration lane composes runtime finality evidence contract-lane policy artifacts and remains fail-closed for missing runtime policy evidence (`Regression: #2101`).
- local KAMN live runtime integration lane forwards explicit runtime provider contract markers into nested runtime policy checks and remains fail-closed on provider-contract drift (`Regression: #2112`).
- local KAMN live runtime integration lane emits fail-closed local-only fast-gate exclusion markers (`ci_fast_gate_eligible=false`, `contracts.ci_fast_gate_scope=local-only`) in summary/policy/docs contracts (`Regression: #2113`).
- live provider operator runbook command/checkpoint/troubleshooting markers remain fail-closed across devnet ops docs and README cross-reference (`Regression: #2114`).
- local live-node validation bundle contract lane and docs parity markers remain fail-closed across devnet ops, CI strategy, and README command surfaces (`Regression: #2134`).
- local KAMN live runtime integration lane requires bounded localhost signed integration prerequisite execution before runtime commit submission (`Regression: #1636`).
- unified local signed-to-Kolme demo lane fails closed for local opt-in, stage prerequisite drift, and runtime budget overruns (`Regression: #1640`).
- unified local signed-to-Kolme demo policy rejects missing runtime submit/finality evidence markers and broken submit/finality linkage (`Regression: #2388`).
- local fork process lifecycle integration lane fails closed for process start/readiness/integration/teardown/budget drift and missing local opt-in (`Regression: #1494`).
- local fork process lifecycle integration lane propagates integration runtime finality pass-through options and artifacts to nested integration command composition (`Regression: #1973`).
- local fork process lifecycle integration lane propagates runtime policy report linkage to nested integration command composition and summary artifact lineage (`Regression: #2104`).
- local fork process lifecycle integration lane propagates rollback/recovery evidence linkage options and deterministic artifact markers in summary/policy contracts (`Regression: #2107`).
- local fork profile preflight lane fails closed for local opt-in, checkout/profile contract drift, probe command failures, and runtime budget overruns (`Regression: #1648`).
- local fork profile preflight policy and contract-lane command/report drift remains fail-closed (`Regression: #1697`).
- local fork self-test lane fails closed for local opt-in, nested matrix/policy checkpoint failures, and runtime budget overruns (`Regression: #1652`).
- local fork self-test policy and contract-lane command/report drift remains fail-closed (`Regression: #1702`).
- local fork portability preflight lane fails closed for local opt-in, mold linker drift, libudev dependency drift, and compile probe failures (`Regression: #1707`).
- local fork checkout bootstrap lane fails closed for local opt-in, checkout provenance drift, diagnostics command failures, and runtime budget overruns (`Regression: #1663`).
- real-fork local process wrapper bootstrap-first prerequisite ordering remains fail-closed for bootstrap lane/policy checkpoint drift (`Regression: #1667`).
- local-only heavy E2E lane checkout-bootstrap contract checkpoint composition remains fail-closed for command/id ordering drift (`Regression: #1677`).
- real-fork local process wrapper lane fails closed for local opt-in, serve-command profile drift, self-test/lifecycle/policy checkpoint failure, and runtime budget overruns (`Regression: #1644`).
- real-fork local process wrapper lane propagates lifecycle runtime finality pass-through options into nested process lifecycle integration command composition (`Regression: #1975`).
- real-fork local process wrapper lane lifecycle mode selector (`--lifecycle-mode`) remains fail-closed for nested process lifecycle command composition drift (`Regression: #1977`).
- real-fork local process wrapper lane propagates lifecycle rollback/recovery evidence pass-through options into nested process lifecycle command composition and summary artifact lineage (`Regression: #2109`).
- real-fork local process wrapper policy checker lane remains fail-closed for schema/contracts/checkpoint drift (`Regression: #1671`).
- local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`).
- local runtime-commit live proof lane preflight health probe and default live-provider ignored-test dispatch remain fail-closed (`Regression: #1829`).
- local runtime-commit live proof lane evidence policy remains fail-closed for missing live-provider command marker contracts (`Regression: #2095`).
- local runtime-commit submit/finality evidence marker policy and contract lane parity remains fail-closed (`Regression: #2099`).
- local native API parity live proof lane fails closed without local opt-in and on nonce/broadcast/finality timeout or command failures (`Regression: #1465`).
- native parity fast/local command matrix docs drift remains fail-closed (`Regression: #1468`).
- local probe fork-info query semantics and native parity broadcast method drift remain fail-closed (`Regression: #1482`).
- block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`).
- localhost two-process signed-demo command/schema markers remain fail-closed across README and Kolme devnet ops docs (`Regression: #1612`).
- localhost signed demo contract-lane status markers remain fail-closed (`Regression: #1609`).
- localhost signed demo/integration report composition remains centralized via shared helper wiring (`Regression: #1617`).
- localhost signed timeout-race handling remains bounded and fail-closed via shared scenario runner retries (`Regression: #1621`).
- localhost signed signature-mismatch bounded retries remain deterministic and fail-closed via shared scenario runner retries (`Regression: #1625`).
- localhost signed replay-nonce bounded retries remain deterministic and fail-closed via shared scenario runner retries (`Regression: #1629`).
- localhost signed admission bounded retries remain deterministic and fail-closed via shared scenario runner retries (`Regression: #1632`).
- Failover/sync budget overruns and unscheduled deep-lane execution fail closed (`Regression: #788`).

## Local Validation

```bash
bash scripts/kolme/test_validate_triadic_devnet_smoke.sh
bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh
bash scripts/kolme/test_run_local_fork_sync_metadata_lane.sh
bash scripts/kolme/test_run_local_fork_smoke_evidence_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh
bash scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh
bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_api_probe_lane.sh
bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh
bash scripts/kolme/test_run_local_kolme_live_api_conformance_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh
bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh
bash scripts/kolme/test_run_local_signed_to_kolme_demo_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_profile_preflight_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_profile_preflight_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_self_test_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_self_test_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_portability_preflight_lane.sh
bash scripts/kolme/test_check_local_kolme_fork_portability_preflight_policy.sh
bash scripts/kolme/test_run_local_kolme_fork_portability_preflight_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_lane.sh
bash scripts/kolme/test_check_local_kolme_fork_checkout_bootstrap_policy.sh
bash scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_contract_lane.sh
bash scripts/kolme/test_check_local_kolme_fork_real_process_policy.sh
bash scripts/kolme/test_run_local_kolme_fork_real_process_contract_lane.sh
bash scripts/kolme/test_run_local_runtime_commit_live_lane.sh
bash scripts/kolme/test_run_local_runtime_commit_live_finality_evidence_contract_lane.sh
bash scripts/kolme/test_run_local_native_api_parity_live_proof_contract_lane.sh
bash scripts/kolme/test_run_fast_gate_native_api_parity_contract_lane.sh
bash scripts/kolme/test_run_block_fallback_reconciliation_contract_lane.sh
bash scripts/kolme/test_run_local_bootstrap_health_checks.sh
bash scripts/kolme/test_check_local_bootstrap_health_policy.sh
bash scripts/kolme/test_run_local_bootstrap_health_checks_contract_lane.sh
bash scripts/kolme/test_run_local_e2e_integration_lane.sh
bash scripts/kolme/test_check_local_e2e_integration_policy.sh
bash scripts/kolme/test_run_local_e2e_integration_contract_lane.sh
bash scripts/kolme/test_run_local_heavy_validation_matrix.sh
bash scripts/kolme/test_check_local_heavy_validation_matrix_policy.sh
bash scripts/kolme/test_run_local_heavy_validation_matrix_contract_lane.sh
bash scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh
bash scripts/ci/test_kolme_wave8_wrapper_family_baseline_contract.sh
bash scripts/ci/test_check_kolme_wave8_wrapper_family_budget_trend.sh
bash scripts/runtime/test_select_failover_sync_drill_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_deep_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_suite.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh
bash scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh
bash scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh
bash scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
