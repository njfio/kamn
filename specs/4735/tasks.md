# Tasks: Issue #4735

Status: Reviewed
Issue: #4735

## Ordered Tasks

T1 (RED):
- Validate extraction boundary and deployment contract assertions against current state and confirm
  failing/drift findings for:
  - block pipeline module follow-through
  - SDK oversized payload selector flake path
  - production limiter-path `expect()`
  - compose TLS/HTTPS healthcheck wiring

T2 (GREEN):
- Repair block pipeline extraction follow-through wiring and support helper ownership.
- Harden SDK TCP sender shutdown behavior for benign peer-close races.
- Remove production service API limiter-path `expect()` calls via deterministic fallback policy.
- Enable compose service API TLS env markers and HTTPS healthchecks for all role services.
- Update deployment docs and contract checks for TLS compose requirements.

T3 (REFACTOR):
- Keep root `block_pipeline.rs` focused on orchestration and module wiring only.
- Keep deployment TLS material guidance in `deploy/certs/README.md` instead of committed key
  artifacts.

T4 (VERIFY):
- Run:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -p kamn-sdk -p kamn-node -- -D warnings`
  - `cargo test -p kamn-core --test p2p_block_module_extraction_contract`
  - `cargo test -p kamn-core --test transport_pipeline_module_extraction_contract`
  - `cargo test -p kamn-core --test block_pipeline_gossip_ingest`
  - `cargo test -p kamn-core --test block_pipeline_transport_fed`
  - `cargo test -p kamn-core --test block_pipeline_canonical_reconciliation`
  - `cargo test -p kamn-sdk --test tcp_transport_adapter`
  - `cargo test -p kamn-node concurrency_limit_is_exceeded`
  - `cargo test -p kamn-node rate_limit_is_exceeded`
  - `cargo test -p kamn-node lifecycle_rejection_projection_is_deterministic`
  - `cargo test -p kamn-node lifecycle_projection_matches_live_concurrency_rejection`
  - `bash scripts/deploy/test_deployment_assets.sh`
  - `bash scripts/deploy/test_validate_deployment_assets_live.sh`
  - `bash scripts/deploy/validate_deployment_assets_live.sh`

## TDD Evidence

- RED command/output:
  - `cargo check -p kamn-core --lib`
    - Failed with unresolved helper ownership/imports in
      `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs`.
  - `rg -n \"expect\\(\" crates/kamn-node/src/service_api_endpoint.rs`
    - Returned production limiter-path `expect()` sites.
  - `rg -n \"http://127.0.0.1:1908\" deploy/docker-compose.yml`
    - Returned plain-HTTP healthcheck markers.

- GREEN command/output:
  - `cargo check -p kamn-core --lib`
    - Passed after helper ownership/wiring fixes.
  - `cargo test -p kamn-sdk --test tcp_transport_adapter integration_tcp_adapter_rejects_oversized_wire_payload -- --exact`
    - Passed on repeated runs.
  - `bash scripts/deploy/test_deployment_assets.sh`
    - Passed: `deployment asset contract tests passed.`

- Regression summary:
  - Extraction and transport-fed block pipeline selectors pass with module boundaries intact.
  - Service API lifecycle limiter behaviors remain fail-closed without production `expect()`.
  - Deployment asset and live validation selectors enforce TLS compose governance.
