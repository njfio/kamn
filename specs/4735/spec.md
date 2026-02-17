# Spec: Issue #4735

Status: Reviewed
Issue: #4735
Parent: #4310
Milestone: specs/milestones/r27-36-deep-validation-hardening-concurrency-safety-and-observability-emission-governance/index.md
Priority: P1

## Problem Statement

Post-extraction and deployment follow-through drift remained in four areas: partial
`block_pipeline` module follow-through, intermittent SDK TCP sender shutdown failures in oversized
payload rejection flow, production middleware `expect()` calls in service API limiter paths, and
compose TLS deployment templates not consuming available HTTPS service API mode.

## Scope

In scope:
- Finalize `block_pipeline` module declaration/re-export wiring and keep extracted boundaries
  compiled and test-verified.
- Preserve deterministic fail-closed behavior in transport and block-pipeline paths.
- Remove production limiter-path `expect()` usage from service API middleware and replace with
  deterministic fallback policy projections.
- Harden SDK TCP sender half-close handling for benign peer-close races after payload rejection.
- Enable TLS mode in compose service API runtime configuration and switch healthchecks to HTTPS,
  including docs and deployment asset contract updates.

Out of scope:
- New wire/protocol formats.
- New runtime mode capabilities.
- mTLS client-auth implementation.

## Acceptance Criteria

AC-1:
Given extracted `block_pipeline` modules exist, when root module compiles and extraction contracts
run, then root declarations/re-exports match extracted ownership boundaries and tests pass.

AC-2:
Given oversized wire payload rejection flow in SDK TCP adapter, when sender writes and peer closes
after rejection, then sender does not fail on benign shutdown races.

AC-3:
Given service API lifecycle limiter projection lookup in middleware, when policy projection is
unavailable or lookup changes, then middleware fails closed without production `expect()` panics.

AC-4:
Given compose deployment templates for processor/listener/approver service APIs, when deployment
assets are validated, then TLS env markers and HTTPS healthchecks are configured and docs reflect
the contract.

## Conformance Cases

- C-01 (AC-1, Regression):
  - Test: `cargo test -p kamn-core --test p2p_block_module_extraction_contract`
  - Expectation: block pipeline extraction contracts pass.

- C-02 (AC-1, Regression):
  - Test: `cargo test -p kamn-core --test transport_pipeline_module_extraction_contract`
  - Expectation: root boundary checks pass for block pipeline transport support ownership.

- C-03 (AC-1, Functional/Integration):
  - Test: `cargo test -p kamn-core --test block_pipeline_transport_fed`
  - Expectation: transport-fed consensus/commit behavior remains stable.

- C-04 (AC-2, Integration/Regression):
  - Test: `cargo test -p kamn-sdk --test tcp_transport_adapter integration_tcp_adapter_rejects_oversized_wire_payload -- --exact`
  - Expectation: oversized payload rejection path passes without sender shutdown failure.

- C-05 (AC-3, Functional/Integration):
  - Test: `cargo test -p kamn-node concurrency_limit_is_exceeded`
  - Expectation: limiter rejection remains fail-closed and stable.

- C-06 (AC-3, Functional/Integration):
  - Test: `cargo test -p kamn-node rate_limit_is_exceeded`
  - Expectation: ingress rate-limiter rejection remains fail-closed and stable.

- C-07 (AC-4, Contract/Docs):
  - Test: `bash scripts/deploy/test_deployment_assets.sh`
  - Expectation: compose TLS markers and HTTPS healthcheck contracts are enforced.

- C-08 (AC-4, Contract/Integration):
  - Test: `bash scripts/deploy/validate_deployment_assets_live.sh`
  - Expectation: live deployment-asset validation stays GO with deterministic markers.

## Success Metrics / Observable Signals

- `crates/kamn-core/src/block_pipeline.rs` remains decomposed with explicit module declarations and
  extraction-contract pass status.
- SDK TCP adapter oversized-payload rejection path is stable across repeated selector runs.
- `crates/kamn-node/src/service_api_endpoint.rs` has no production limiter-path `expect()`.
- Compose deployment contracts assert TLS env markers and HTTPS healthchecks for all role services.
