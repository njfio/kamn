# Kolme Live Integration Architecture

This document captures the live-node integration contract surface between KAMN
runtime signing and `njfio/kolme_fork` compatibility expectations.

## Composed Full-Stack E2E Boundary (Task #3433)

- Architecture boundary:
  - `scripts/runtime/validate_local_full_stack_integration_live.sh` is the top-level composed runtime lane.
  - run-mode composition includes:
    - `scripts/runtime/validate_full_io_scenario_matrix_live.sh`
    - `scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh`
    - `scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh`
    - `scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh`
- Evidence lineage:
  - top-level summary schema: `kamn.runtime.local-full-stack-integration-live-report.v1`
  - nested Kolme summary schema: `kamn.kolme.local-kamn-live-runtime-integration-summary.v1`
  - nested Kolme policy schema: `kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1`
  - composed evidence bundle schema: `kamn.runtime.local-full-stack-integration-evidence-bundle.v1`
  - release manifest linkage: `scripts/runtime/release_evidence_manifest.json` requires artifact id `local_full_stack_integration`.
- Marker taxonomy (top-level fail-closed contract):
  - `transport_convergence_status`
  - `libp2p_process_isolation_status`
  - `libp2p_two_node_process_isolated_status`
  - `libp2p_three_node_process_isolated_status`
  - `signer_provenance_status`
  - `runtime_commit_submission_status`
  - `runtime_commit_finality_status`
  - `local_heavy_runtime_budget_status`
  - `elapsed_seconds`
  - `max_seconds`
  - `command_max_seconds`
  - `combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1`
  - `combined_transport_reason_codes=fork_choice_stale_block_height`
  - `combined_kolme_runtime_reason_code`
  - `kolme_runtime_commit_failure_taxonomy_version=v1`
  - `kolme_runtime_commit_failure_taxonomy`
  - `kolme_fixture_profile=real-node-non-synthetic-v1`
  - `kolme_fixture_profile_version=v1`
  - `kolme_fixture_profile_status`
  - `runtime_provider_contract_status`
  - `runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider`
  - `runtime_signing_profile=kolme-fork-secp256k1-v1`
  - `runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1`
- Deterministic fail-closed reasons:
  - `local_full_stack_integration_policy_reason_taxonomy_version_mismatch`
  - `local_full_stack_integration_policy_libp2p_process_isolation_status_mismatch`
  - `local_full_stack_integration_policy_libp2p_two_node_process_isolated_status_mismatch`
  - `local_full_stack_integration_policy_libp2p_three_node_process_isolated_status_mismatch`
  - `local_full_stack_integration_policy_libp2p_summary_three_node_partition_rejoin_status_mismatch`
  - `local_full_stack_integration_policy_libp2p_summary_three_node_publish_drop_status_mismatch`
  - `local_full_stack_integration_policy_runtime_budget_status_mismatch`
  - `local_full_stack_integration_policy_runtime_budget_exceeded`
  - `local_full_stack_integration_policy_kolme_summary_schema_mismatch`
  - `local_full_stack_integration_policy_kolme_policy_final_decision_mismatch`
  - `release_manifest_missing_required_artifact:local_full_stack_integration`
- Operator references:
  - CI policy surface: `docs/ci/strategy.md`
  - Local runbook: `docs/planning/kolme-devnet-ops.md`
  - Release gate command: `bash scripts/runtime/run_go_no_go_gate_lane.sh --mode dry-run --output-json /tmp/go-no-go-gate-report.json`

## Signature Parity Lane (Task #2299)

- Vector source policy:
  - `fixtures/kolme_commit/signature_parity_vectors.json`
  - schema: `kamn.kolme.signature-parity-vectors.v1`
  - source contract marker: `njfio/kolme_fork-secp256k1-v1`
- Parity matrix runner:
  - `python3 scripts/kolme/run_signature_parity_matrix.py --fixture fixtures/kolme_commit/signature_parity_vectors.json --output-json /tmp/kolme-signature-parity-matrix-report.json`
  - report schema: `kamn.kolme.signature-parity-matrix-report.v1`
  - runner executes deterministic adapter probe:
    - `KAMN_KOLME_LOCAL_HEAVY=1 cargo test -p kamn-node integration_kolme_live_signer_vector_probe_contract -- --nocapture`
- Policy checker:
  - `python3 scripts/kolme/check_signature_parity_policy.py --report-file /tmp/kolme-signature-parity-matrix-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-signature-parity-policy-report.json`
  - policy schema: `kamn.kolme.signature-parity-policy-report.v1`
- Contract lane wrapper:
  - `bash scripts/kolme/run_signature_parity_contract_lane.sh --output-json /tmp/kolme-signature-parity-matrix-report.json --policy-output-json /tmp/kolme-signature-parity-policy-report.json`

## Drift Handling

- Known-good vectors must produce `GO` parity outcomes.
- Known-bad vectors must produce `NO-GO` outcomes with deterministic reason codes
  (for example `parity_signature_mismatch`).
- Any probe failure without explicit mismatch reasons is classified as
  `parity_probe_failed` and treated as fail-closed.

## Managed Signer Routing (Task #2323)

- Strict managed signer mode:
  - `--kolme-live-strict-signer-contracts --kolme-live-signer-key-source managed-external`
  - key-reference env markers:
    - `KAMN_KOLME_LIVE_SIGNER_KEY_REF` (`ops-primary`)
    - `KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY` (`ops-secondary`)
- Runtime contracts:
  - secure-provider handshake routing is enforced via signer backend contracts before
    payload signing.
  - malformed/missing key-reference markers fail closed.
  - raw private-key env markers are forbidden in managed-external mode.
- Deterministic reason-code classes for managed-external secure path:
  - `managed_signer_provider_unavailable`
  - `managed_signer_provider_handshake_rejected`
  - `managed_signer_backend_error`
  - `managed_signer_raw_private_key_forbidden`

## Structured Runtime Logging Taxonomy (Task #4121)

- Bootstrap logging contract:
  - runtime logs normalize `correlation_id` and `reason_code` fields.
  - when omitted, deterministic fallback markers are projected as `none`.
- Retry taxonomy contract:
  - nonce, submit, and finality retry markers emit both `reason` and
    canonical `reason_code`.
  - `reason` and `reason_code` values must match for retry-class events.
- Correlation contract:
  - submit/finality lifecycle markers share request-idempotency
    correlation IDs.
  - nonce retry markers project a deterministic pubkey-scoped correlation
    marker (`kolme.live.nonce:<pubkey>`).
- Terminal decision contract:
  - retry terminal markers include `terminal_decision` and canonical
    `reason_code`.
  - finality degraded outcomes map `reason_code` to the final resolution
    marker (`finality-unavailable`, `finality-timeout`, etc.).

Validation commands:

- `cargo test -p kamn-node functional_kolme_live_retry_emits_structured_retry_markers -- --nocapture`
- `cargo test -p kamn-node functional_kolme_live_nonce_retry_emits_structured_retry_marker -- --nocapture`
- `cargo test -p kamn-node retry_exhaustion_emits_terminal_decision_marker -- --nocapture`
