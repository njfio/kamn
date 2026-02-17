# Plan — #4267 Protocol Taxonomy + Runbook Marker Parity

Status: Reviewed

## Approach

1. Extend service api axum contract lane orchestration to validate runbook parity markers.
2. Define deterministic fail-closed mismatch categories for:
   - taxonomy mapping drift
   - runbook marker parity mismatch
3. Add red/green regression checks in lane tests for both mismatch categories.
4. Document parity marker contract in deploy compatibility and release checklist docs.
5. Add docs-contract tests for new parity marker sections.

## Affected Modules

- `scripts/runtime/service_api_contract_lane_runner.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks + Mitigations

- Risk: broad runner changes could break other lanes.
  - Mitigation: keep checks optional and enabled only when lane variables are set.
- Risk: brittle doc-marker assertions.
  - Mitigation: constrain to deterministic marker lines only.

## Interfaces / Contracts

- New optional runner variables:
  - `RUNBOOK_DOC`
  - `RUNBOOK_REQUIRED_MARKERS`
  - `RUNBOOK_TAXONOMY_DRIFT_REASON_CODE`
  - `RUNBOOK_MARKER_PARITY_REASON_CODE`
- Deterministic reason outputs:
  - `protocol_taxonomy_mapping_drift_detected`
  - `runbook_marker_parity_mismatch`

## ADR

Not required (no new dependency, architecture, or protocol change).
