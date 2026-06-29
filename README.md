# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous agents.

This repository contains the Rust runtime/core crates, live-node scaffolding, SDK surfaces, and CI/governance tooling used to evolve the protocol safely.

## What This Repository Contains

- `crates/kamn-core`: protocol/domain logic and contract suites
- `crates/kamn-node`: node runtime entrypoint and service API
- `crates/kamn-sdk`: Rust SDK client surface
- `crates/kamn-agent-lib`: agent-facing auth/identity helpers
- `crates/kamn-kolme`: Kolme live provider integration layer
- `scripts/`: CI, contract lanes, and deterministic validation utilities
- `docs/`: architecture, operations, security, and planning references

## Quickstart

Prerequisites:
- Rust toolchain (`cargo`, `rustc`)
- Bash
- Python 3
- Node.js/npm (for dashboard/TS-related lanes)

Core validation:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
cargo test -p kamn-node
bash scripts/ci/test_ci_tools.sh
```

Live HTTPS dependency posture checks (`kamn-core`):
- Dependencies:
- `rustls`
- `rustls-pemfile`
- `webpki-roots`

```bash
cargo check -p kamn-core --features live-https
cargo check -p kamn-core --no-default-features
```

Fast repository lanes:

```bash
make check
make test
make ci-tools
```

Live-network smoke entrypoints:

```bash
make smoke-live-network
make deep-live-network
make demo-localhost-transport
```

## Workflow

1. Open/confirm issue + milestone scope.
2. Author `specs/<issue-id>/{spec,plan,tasks}.md`.
3. Run RED -> GREEN -> REFACTOR for each task.
4. Update docs/specs in the same PR when behavior changes.
5. Keep `cargo fmt`, `clippy`, and scoped tests green before push.

Repository process contract:
- `AGENTS.md`

## Architecture Map

Start here for system navigation:
- `docs/architecture/README.md`

Core architecture references:
- `docs/architecture/runtime-layout.md`
- `docs/architecture/service-runtime.md`
- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/kamn-node-module-map.md`
- `docs/foundation/kolme-runtime-architecture.md`
- `docs/foundation/runtime-network.md`
- `docs/architecture/adr-kamn-core-live-tls-transport.md`

## Contract Reference

Detailed command matrices, contract markers, policy snippets, and lane-specific references are maintained in:

- `docs/developer/readme-contract-reference.md`

This keeps the root README onboarding-focused while preserving deterministic contract markers in a stable docs location.

Managed-signer backend SLO telemetry anchors:
- `generate_managed_signer_backend_slo_telemetry_bundle.sh`
- `run_managed_signer_backend_slo_telemetry_contract_lane.sh`
- `kamn.kolme.managed-signer-backend-slo-telemetry.v1`
- `managed_signer_backend_timeout_rate_threshold_exceeded`
- `managed_signer_backend_unavailable_rate_threshold_exceeded`
- `managed_signer_backend_error_rate_threshold_exceeded`
- `managed_signer_backend_ci_fast_gate_failed`
- `signer_key_source=managed-external`
- `contracts.required_signer_key_source=managed-external`

Managed-signer backend SLO policy anchors:
- `check_managed_signer_backend_slo_policy.py`
- `run_managed_signer_backend_slo_policy_contract_lane.sh`
- `kamn.kolme.managed-signer-backend-slo-policy-report.v1`
- `kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1`
- `managed_signer_backend_slo_within_threshold`
- `managed_signer_backend_no_action_required`
- `managed_signer_backend_reduce_timeout_burst`
- `managed_signer_backend_failover_endpoint`
- `managed_signer_backend_enable_circuit_breaker`
- `managed_signer_backend_replay_ci_fast_gate`

Managed-signer startup validation anchors:
- `run_managed_signer_startup_live_validation_contract_lane.sh`
- `kamn.kolme.managed-signer-startup-live-validation-contract-report.v1`
- `deployment_preflight_passed`
- `signer_rotation_promotion_stalled`
- `quorum_evidence_custody_sha256_mismatch`
- `checkpoint_failed_signer_profile_contract`
- `checkpoint_failed_signer_provenance_contract`
- `checkpoint_failed_signer_rotation_freshness_contract`
- `signer_key_source_production_managed_external_required`
- `signer_profile_mismatch`
- `signer_rotation_epoch_stale`
- `managed_signer_rotation_promotion_stalled_fail_closed_status=verified`
- `managed_signer_custody_audit_parity_fail_closed_status=verified`
- `managed_signer_rotation_reason_taxonomy_status=verified`
- `managed_signer_rehearsal_output_normalization_status=verified`
- `managed_signer_rotation_reason_taxonomy_version=kamn.kolme.managed-signer-startup-reason-taxonomy.v1`
- `managed_signer_rotation_reason_codes_csv=custody_continuity_bypass_detected,quorum_evidence_custody_sha256_mismatch,signer_rotation_epoch_stale,signer_rotation_promotion_stalled,signer_rotation_rehearsal_drift_detected`
- `ci_local_promotion_budget_boundary_status=verified`
- `execution_scope=local-scheduled`

Live-provider runtime integration anchors:
- `run_local_live_provider_runtime_integration_contract_lane.sh`
- `run_local_runtime_commit_live_lane.sh`
- `check_local_runtime_commit_live_evidence_policy.py`
- `provider_client_contract=KolmeRuntimeCommitLiveProvider`
- `provider_client_contract_mismatch`
- `provider_in_memory_reference_detected`
- `provider_signer_adapter_contract=KolmeForkSecp256k1SignerAdapter`
- `provider_signer_adapter_contract_mismatch`
- `live_preflight_failed`
- `live_preflight_timeout`

Localhost signed integration anchors:
- `run_localhost_signed_integration_contract_lane.sh`
- `check_localhost_signed_integration_evidence_policy.sh`
- `/tmp/localhost-signed-integration-contract-report.json`

## Key Links

- CI strategy: `docs/ci/strategy.md`
- Engineering hardening wave: `docs/planning/engineering-hardening-wave.md`
- Runtime/live ops: `docs/planning/live-network-wave.md`
- Kolme devnet ops: `docs/planning/kolme-devnet-ops.md`
- Rustdoc publishing guide: `docs/developer/rustdoc-publishing.md`
- Missing-docs policy checker: `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- Security guidance: `docs/security/secure-coding.md`
- TLS hardening: `docs/security/tls-hardening.md`
