# Milestone: R26.2 Service Edge Hardening

- Milestone ID: `r26-2-service-edge-hardening`
- GitHub Milestone: `#30`
- Milestone Title: `R26.2 Service edge hardening (TLS + axum + signer decomposition)`
- Scope Status: `In progress`

## Objective
Deliver service-edge hardening by converging TLS activation, API/observability endpoint hardening, and signer monolith decomposition into auditable module boundaries.

## Issue Map
- Story `#3628`: decompose signer monolith into modular signing adapter and policy layers
  - Task `#3636`: extract signer adapter module for crypto and key-source operations
  - Task `#3637`: extract signer policy module for profile normalization and quorum checks
    - Subtask `#3654`: extract signer_policy with deterministic quorum and profile checks
    - Subtask `#3807`: add signer_policy reason-taxonomy drift and docs parity contracts
  - Task `#3638`: deliver signer parity harness and migration completion

## Contract Signals
- Signer responsibilities are isolated into explicit modules with deterministic error reason codes.
- Managed signer backend control and signer policy contracts fail closed under malformed input.
- Runtime signing parity is preserved across env-local and managed-external paths.
- Deployment TLS and service endpoint contracts remain documented and testable.

## Verification Surface
- `cargo test -p kamn-node signer -- --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
- `bash scripts/deploy/test_deployment_assets.sh`
