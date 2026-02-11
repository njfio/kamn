# Kolme Triadic Devnet Operability Plan (Issues #784, #785, #787, #788, #1405, #1417, #1418, #1501)

This plan defines the deterministic, low-cost local smoke contract for triadic
runtime roles (processor/listener/approver) and its CI-compatible validation.

The live backend contract inventory for `njfio/kolme_fork` is tracked in:
- `docs/research/kolme-fork-api-contract-inventory.md`

## Scope

- One-command triadic devnet smoke orchestration.
- Deterministic marker validation from fixture contract.
- PR-safe runtime budget guard for smoke lane cost control.

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
  - smoke command timeout/exceeded budget is reported as `fork_smoke_command_timeout`.

## Local-Only Fork Rust Test Matrix Lane (Issue #1537)

- Local fork Rust test matrix runner:
  - `bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json`
- Explicit local-only Rust test matrix execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json`
- Summary schema:
  - `kamn.kolme.local-fork-rust-test-matrix-summary.v1`
- Deterministic checkpoints include:
  - `run_local_fork_sync_metadata_lane.sh` metadata validation prior to Rust command execution.
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
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Explicit local-only live runtime integration execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 210 --bootstrap-max-seconds 90 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Summary schema:
  - `kamn.kolme.local-kamn-live-runtime-integration-summary.v1`
- Deterministic checkpoints include:
  - `run_local_kolme_fork_bootstrap_readiness_lane.sh` run-mode validation for pinned checkout provenance and API readiness.
  - `run_local_kolme_live_api_conformance_harness.sh` run-mode validation for health/query/nonce/broadcast command contracts.
  - explicit runtime-commit submit-profile probe over `PUT /broadcast` with fail-closed reason codes.
  - signed runtime-commit envelope translation enforces `signer_key_id` presence and canonical message/signature binding before broadcast normalization.
  - finality verification uses `/notifications` first with bounded `/block/{height}` fallback; no runtime commit status endpoint dependency.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 210 seconds with per-stage budget caps.
  - local KAMN live runtime integration run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Kolme Fork Process Lifecycle Integration Lane (Issue #1494)

- Local fork process lifecycle runner:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Explicit local-only process lifecycle execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --max-seconds 300 --startup-max-seconds 45 --integration-max-seconds 240 --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json`
- Summary schema:
  - `kamn.kolme.local-fork-process-lifecycle-summary.v1`
- Deterministic checkpoints include:
  - process command orchestration: start -> readiness probe -> nested `run_local_kamn_live_runtime_integration_lane.sh` -> teardown.
  - readiness contract over `GET /healthz` and `GET /fork-info?chain_version=<version>`.
  - fail-closed reason codes for local opt-in, serve-command, bootstrap, readiness, integration, teardown, and budget drift.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - lane default budget is bounded to 300 seconds with per-stage integration budget caps.
  - local fork process lifecycle integration run-mode execution remains excluded from PR fast-gate workflow routing.

## Local Runtime Commit Live Proof Lane (Issue #1450)

- Local runtime-commit live lane runner:
  - `bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode dry-run --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Explicit opt-in live execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --live-command "cargo test -p kamn-core --test kolme_runtime_commit_http_transport" --max-seconds 90 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Summary schema:
  - `kamn.kolme.local-runtime-commit-live-summary.v1`
- Deterministic checkpoints include:
  - explicit local-only opt-in marker (`KAMN_KOLME_LOCAL_HEAVY=1`)
  - bounded live command timeout via `--max-seconds`
  - machine-readable pass/fail reason codes for missing opt-in, command failure, and timeout
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - live command timeout/exceeded budget is reported as `live_runtime_commit_command_timeout`.

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

## Deterministic Local Bootstrap Health Checks (Issue #1417)

- Bootstrap health-check runner:
  - `bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode dry-run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Explicit opt-in bootstrap execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Summary schema:
  - `kamn.kolme.local-bootstrap-summary.v1`
- Deterministic readiness checks include:
  - `validate_version_compatibility.py`
  - `generate_fork_compatibility_evidence.py`
  - `check_fork_compatibility_policy.py`
  - `run_triadic_devnet_smoke.sh`
  - `validate_triadic_devnet_smoke.py`
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.

## Local-Only Heavy End-to-End Lane (Issue #1418)

- Local-only E2E lane runner:
  - `bash scripts/kolme/run_local_e2e_integration_lane.sh --mode dry-run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Explicit opt-in E2E execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_e2e_integration_lane.sh --mode run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Summary schema:
  - `kamn.kolme.local-e2e-integration-summary.v1`
- Deterministic checkpoints include:
  - `run_local_bootstrap_health_checks.sh`
  - `run_runtime_commit_adapter_contract_lane.sh`
  - `run_live_transport_parity_contract_lane.sh --languages python,typescript`
- Cost policy:
  - lane enforces explicit local-only opt-in and a deterministic runtime budget ceiling.

## Local-Only Heavy Kolme Validation Matrix (Issue #1405)

- Local-only matrix runner:
  - `bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Explicit opt-in execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Summary schema:
  - `kamn.kolme.local-heavy-validation-summary.v1`
- Heavy command set includes:
  - `scripts/kolme/run_local_bootstrap_health_checks.sh`
  - `scripts/kolme/run_version_compatibility_replay_deep_lane.sh`
- Cost policy:
  - matrix execution remains local-only and is excluded from PR fast-gate workflow routing.

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
- local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`).
- local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`).
- local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`).
- local fork Rust test matrix lane fails closed on missing local opt-in, metadata sync drift, and per-command timeout/failure paths (`Regression: #1537`).
- local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`).
- local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`).
- local live API conformance harness fails closed for probe/native parity prerequisite failures, runtime budget overruns, and endpoint contract drift (`Regression: #1483`).
- local fork bootstrap/readiness lane fails closed for sync/probe prerequisite failures, runtime budget overruns, and missing local opt-in (`Regression: #1488`).
- local KAMN live runtime integration lane fails closed for bootstrap/conformance/runtime-commit prerequisite drift, runtime budget overruns, and missing local opt-in (`Regression: #1489`).
- local fork process lifecycle integration lane fails closed for process start/readiness/integration/teardown/budget drift and missing local opt-in (`Regression: #1494`).
- local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`).
- local native API parity live proof lane fails closed without local opt-in and on nonce/broadcast/finality timeout or command failures (`Regression: #1465`).
- native parity fast/local command matrix docs drift remains fail-closed (`Regression: #1468`).
- local probe fork-info query semantics and native parity broadcast method drift remain fail-closed (`Regression: #1482`).
- block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`).
- Failover/sync budget overruns and unscheduled deep-lane execution fail closed (`Regression: #788`).

## Local Validation

```bash
bash scripts/kolme/test_validate_triadic_devnet_smoke.sh
bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh
bash scripts/kolme/test_run_local_fork_sync_metadata_lane.sh
bash scripts/kolme/test_run_local_fork_smoke_evidence_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh
bash scripts/kolme/test_run_local_kolme_api_probe_lane.sh
bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh
bash scripts/kolme/test_run_local_kolme_live_api_conformance_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh
bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh
bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh
bash scripts/kolme/test_run_local_runtime_commit_live_lane.sh
bash scripts/kolme/test_run_local_native_api_parity_live_proof_contract_lane.sh
bash scripts/kolme/test_run_fast_gate_native_api_parity_contract_lane.sh
bash scripts/kolme/test_run_block_fallback_reconciliation_contract_lane.sh
bash scripts/kolme/test_run_local_bootstrap_health_checks.sh
bash scripts/kolme/test_run_local_e2e_integration_lane.sh
bash scripts/kolme/test_run_local_heavy_validation_matrix.sh
bash scripts/runtime/test_select_failover_sync_drill_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_deep_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_suite.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
