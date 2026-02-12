# Kolme Triadic Devnet Operability Plan (Issues #784, #785, #787, #788, #1405, #1417, #1418, #1501)

This plan defines the deterministic, low-cost local smoke contract for triadic
runtime roles (processor/listener/approver) and its CI-compatible validation.

The live backend contract inventory for `njfio/kolme_fork` is tracked in:
- `docs/research/kolme-fork-api-contract-inventory.md`

## Scope

- One-command triadic devnet smoke orchestration.
- Deterministic marker validation from fixture contract.
- PR-safe runtime budget guard for smoke lane cost control.

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
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - run mode remains local/manual and is excluded from PR fast-gate workflow routing.

## Deterministic Local Kolme API Probe Lane (Issue #1439)

- Local API probe runner:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-api-probe-summary.json`
- Active local API probe:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 30 --output-json /tmp/kolme-local-api-probe-summary.json`
- Summary schema:
  - `kamn.kolme.local-api-probe-summary.v1`
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
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds 210 --bootstrap-max-seconds 90 --localhost-signed-max-seconds 45 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Optional runtime finality pass-through to nested live runner:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-finality-command "printf 'finality=final\n'" --runtime-commit-finality-max-seconds 15 --runtime-commit-finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Summary schema:
  - `kamn.kolme.local-kamn-live-runtime-integration-summary.v1`
- Deterministic checkpoints include:
  - `run_local_kolme_fork_bootstrap_readiness_lane.sh` run-mode validation for pinned checkout provenance and API readiness.
  - `run_localhost_signed_integration_contract_lane.sh` run-mode validation of signed message admission/replay guards before Kolme runtime commit execution.
  - `run_local_kolme_live_api_conformance_harness.sh` run-mode validation for health/query/nonce/broadcast command contracts.
  - `run_local_runtime_commit_live_finality_evidence_contract_lane.sh` is composed as the default runtime-commit endpoint step (no raw curl fallback by default).
- optional runtime finality pass-through (`--runtime-commit-finality-command`, `--runtime-commit-finality-max-seconds`, `--runtime-commit-finality-output-file`) and `--runtime-commit-live-policy-report` are wired through to nested runtime finality evidence contract composition.
- runtime provider contract marker (`--runtime-provider-client-contract`) remains explicit and fail-closed for `KolmeRuntimeCommitLiveProvider`.
  - explicit runtime-commit submit-profile probe over `PUT /broadcast` with fail-closed reason codes.
  - signed runtime-commit envelope translation enforces `signer_key_id` presence and canonical message/signature binding before broadcast normalization.
  - finality verification uses `/notifications` first with bounded `/block/{height}` fallback; no runtime commit status endpoint dependency.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 210 seconds with per-stage budget caps.
  - local KAMN live runtime integration run-mode execution remains excluded from PR fast-gate workflow routing.

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
  - `run_local_kamn_live_runtime_integration_contract_lane.sh` run-mode verification before final GO decision.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane enforces bounded per-stage and total runtime budgets.
  - run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Kolme Fork Process Lifecycle Integration Lane (Issue #1494)

- Local fork process lifecycle runner:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Explicit local-only process lifecycle execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --max-seconds 300 --startup-max-seconds 45 --integration-max-seconds 240 --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
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
  - `bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json`
- Explicit local-only checkout bootstrap execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py --report-file /tmp/kolme-local-fork-checkout-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json --policy-output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-checkout-bootstrap-summary.v1`
- Deterministic checkpoints include:
  - checkout preparation (clone/update) against pinned `fork-remote-url`.
  - nested `run_local_fork_sync_metadata_lane.sh` run-mode validation for remote/ref provenance.
  - deterministic diagnostics capture for `git --version`, `cargo --version`, and `rustc --version`.
  - fail-closed reason codes for missing local opt-in, checkout bootstrap failure, metadata drift, diagnostics failure, and runtime budget overrun.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget remains bounded to 120 seconds.
  - run-mode execution remains local-only and excluded from PR fast-gate workflow routing.

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
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --skip-preflight --live-command "printf 'status=submitted\\n'" --finality-command "printf 'finality=final\\n'" --finality-max-seconds 15 --max-seconds 90 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt --finality-output-file /tmp/kolme-local-runtime-commit-live-finality-output.txt`
- Evidence policy checker command:
  - `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-runtime-commit-live-policy.json`
- Finality evidence contract lane command:
  - `bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json`
- Default live-provider smoke command executed by run mode:
  - `cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint`
- Summary schema:
  - `kamn.kolme.local-runtime-commit-live-summary.v1`
  - policy schema: `kamn.kolme.local-runtime-commit-live-policy-report.v1`
- Deterministic checkpoints include:
  - bounded preflight probe against `<base-url>/healthz` before live submit execution (unless `--skip-preflight` is explicitly set)
  - explicit local-only opt-in marker (`KAMN_KOLME_LOCAL_HEAVY=1`)
  - bounded live command timeout via `--max-seconds`
  - optional finality command timeout bound via `--finality-max-seconds`
  - submit/finality evidence marker fields (`submit_evidence_marker_present`, `finality_evidence_marker_present`) remain fail-closed in policy checks
  - machine-readable pass/fail reason codes for missing opt-in, preflight failure/timeout, command failure, and command timeout
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - live command timeout/exceeded budget is reported as `live_runtime_commit_command_timeout`.
  - preflight failures are reported as `live_preflight_failed` or `live_preflight_timeout`.

## Local Native API Parity Live Proof Lane (Issue #1465)

- Native API parity live-proof lane runner:
  - `bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode dry-run --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json`
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
- Bounded contract lane (dry-run + policy):
  - `bash scripts/kolme/run_local_heavy_validation_matrix_contract_lane.sh --output-json /tmp/kolme-local-heavy-validation-summary.json --policy-output-json /tmp/kolme-local-heavy-validation-policy.json`
- Summary schema:
  - `kamn.kolme.local-heavy-validation-summary.v1`
  - policy schema: `kamn.kolme.local-heavy-validation-policy-report.v1`
- Heavy command set includes:
  - `scripts/kolme/run_local_bootstrap_health_checks.sh`
  - `scripts/kolme/run_version_compatibility_replay_deep_lane.sh`
  - `scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
  - `scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh`
  - shared JSON summary generation via `scripts/framework/generate_local_lane_summary.py`
- Cost policy:
  - matrix execution remains local-only and is excluded from PR fast-gate workflow routing.
  - shared opt-in enforcement helper: `scripts/framework/assert_local_heavy_opt_in.sh`.

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
- local KAMN live runtime integration lane requires bounded localhost signed integration prerequisite execution before runtime commit submission (`Regression: #1636`).
- unified local signed-to-Kolme demo lane fails closed for local opt-in, stage prerequisite drift, and runtime budget overruns (`Regression: #1640`).
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
