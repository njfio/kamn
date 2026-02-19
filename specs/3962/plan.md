# Issue #3962 Plan

- Issue: #3962
- Status: Completed
- Spec: `specs/3962/spec.md`

## Implementation Approach
1. Add a new Rust docs-contract test suite for deployment hardening CI dry-run governance and runbook marker parity.
2. RED: require explicit `#3962` closure markers and runbook-parity markers that are not yet present in docs.
3. GREEN: add minimal deterministic markers to `docs/ci/strategy.md` and `docs/plans/2026-02-14-production-service-next-steps.md`.
4. REGRESSION: rerun targeted suite and existing deployment hardening contract suite.

## Affected Modules
- `crates/kamn-core/tests/deployment_hardening_ci_dry_run_contract.rs`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`

## Risks and Mitigations
- Risk: brittle prose assertions.
  - Mitigation: check deterministic marker keys/commands/taxonomy constants, not narrative prose.
- Risk: marker duplication drift across strategy and plan docs.
  - Mitigation: enforce explicit chain/guard markers and reason-code CSV parity in one Rust suite.

## Contracts and Interfaces
- Required CI strategy governance markers:
  - `deployment preflight signer/runtime checks remain fast and ci-fast-gate eligible.`
  - `run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
  - `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Required runbook-parity policy markers:
  - `deployment_preflight_runbook_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-runbook-reason-taxonomy.v1`
  - `deployment_preflight_runbook_reason_codes_csv=deployment_preflight_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch`
- Required closure markers:
  - `deployment_hardening_ci_dry_run_contract_chain=#3950->#3954->#3962`
  - `deployment_hardening_ci_dry_run_contract_guard_command=cargo test -p kamn-core --test deployment_hardening_ci_dry_run_contract -- --nocapture`

## Verification Strategy
- RED: run new contract test before adding docs markers.
- GREEN: add docs markers and rerun new suite.
- REGRESSION: rerun existing deployment hardening lane contract suite.
