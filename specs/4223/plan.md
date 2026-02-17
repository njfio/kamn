# Plan — #4223

Status: Reviewed

## Approach

- Extend `service_api_axum_ingress_live_contract.py` with a `check-evidence-convergence` command and deterministic convergence taxonomy.
- Add wrapper script for convergence checker command surface.
- Integrate convergence checker execution into `validate_service_api_axum_ingress_live_contract_lane.sh`.
- Add dedicated convergence tests and update contract-lane tests for new markers.
- Update CI/docs contract surfaces and docs parity assertions.

## Affected Areas

- `scripts/runtime/service_api_axum_ingress_live_contract.py`
- `scripts/runtime/check_service_api_axum_ingress_live_evidence_convergence.sh`
- `scripts/runtime/test_check_service_api_axum_ingress_live_evidence_convergence.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: reason-taxonomy drift between checker implementation, lane output, and docs.
  - Mitigation: centralize constants in checker and assert exact markers via docs-contract tests.
- Risk: lane integration drift due shared runner usage.
  - Mitigation: keep shared runner unchanged and layer convergence orchestration in the axum lane wrapper (same approach as websocket lane).
