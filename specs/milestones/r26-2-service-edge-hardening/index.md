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
    - Subtask `#3653`: extract signer_adapter for key-source and crypto paths
    - Subtask `#3811`: enforce signer_adapter API boundary and re-export drift contracts
  - Task `#3637`: extract signer policy module for profile normalization and quorum checks
    - Subtask `#3654`: extract signer_policy with deterministic quorum and profile checks
    - Subtask `#3807`: add signer_policy reason-taxonomy drift and docs parity contracts
  - Task `#3638`: deliver signer parity harness and migration completion
    - Subtask `#3766`: add signer migration parity matrix and legacy-behavior diff guard
    - Subtask `#3808`: add signer extraction threshold and ownership budget guards
- Story `#3911`: harden signer key-material lifecycle with zeroization and secret policy enforcement
  - Task `#3912`: zeroize signer key decode/loading intermediates across runtime profiles
    - Subtask `#3913`: add explicit zeroization to signer key decode and transient buffers
    - Subtask `#3914`: add regression checks for signer secret redaction and decode-failure hygiene
  - Task `#3915`: enforce signer secret-lifecycle policy and docs parity contracts
    - Subtask `#3916`: add fail-closed policy checks for fallback signer keys and lifecycle markers
    - Subtask `#3917`: add docs-contract parity checks for signer secret-lifecycle markers

## Contract Signals
- Signer responsibilities are isolated into explicit modules with deterministic error reason codes.
- Managed signer backend control and signer policy contracts fail closed under malformed input.
- Runtime signing parity is preserved across env-local and managed-external paths.
- Deployment TLS and service endpoint contracts remain documented and testable.

## Verification Surface
- `cargo test -p kamn-node signer -- --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
- `bash scripts/deploy/test_deployment_assets.sh`
