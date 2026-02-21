# Issue #5513 Spec - Service API Cross-Store Replay Metrics Exposure

- Status: Accepted
- Issue: #5513
- Parent: #3812
- Milestone: R50.22 Service API cross-store replay telemetry exposure

## Problem Statement
`kamn-core` exposes deterministic cross-store replay taxonomy metadata, but `kamn-node` service API `/metrics` does not surface those markers for runtime observability consumers.

## Scope
In scope:
- Extend `ServiceApiSnapshot` with deterministic cross-store replay taxonomy metadata derived from `kamn-core` APIs.
- Emit additive `/metrics` lines for taxonomy version and reason-code count.
- Add/adjust service API endpoint tests for the new metrics contract.

Out of scope:
- New dependencies.
- Protocol/schema changes outside additive metrics lines.

## Acceptance Criteria
- AC-1: `build_service_api_snapshot` includes cross-store replay reason taxonomy version and reason-code count.
- AC-2: `/metrics` output includes deterministic cross-store replay taxonomy/version and reason-code-count lines.
- AC-3: Service API endpoint tests enforce the new metrics lines for both unknown and daemon observability paths.
- AC-4: Targeted tests pass.

## Conformance Cases
- C-01 (AC-1): snapshot projects taxonomy version from `cross_store_replay_reason_taxonomy_version()`.
- C-02 (AC-1): snapshot projects deterministic reason-code count from `cross_store_replay_reason_codes_csv()`.
- C-03 (AC-2): metrics response includes `kamn_service_api_cross_store_replay_reason_taxonomy_info{version="..."} 1`.
- C-04 (AC-2): metrics response includes `kamn_service_api_cross_store_replay_reason_code_count <n>`.
- C-05 (AC-3): tests verify metrics lines for API-mode (unknown observability) and daemon-mode snapshots.
- C-06 (AC-4): targeted `kamn-node` test slice passes.

## Success Metrics / Observable Signals
- Runtime `/metrics` consumers can ingest cross-store replay policy taxonomy metadata without shell lane parsing.
- CI-enforced service API tests fail on marker drift.
