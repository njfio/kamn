# Plan: Issue #4433

Status: In Progress
Issue: #4433

## Approach

1. Add RED assertions in existing compose-topology lane and policy tests for packaging drift and
   missing taxonomy/evidence markers.
2. Implement deterministic packaging taxonomy surfaces in lane summary and policy output.
3. Enforce explicit fail-closed reason-code mapping for compose/manifest/config drift classes.
4. Update deploy/CI docs to pin new taxonomy and deterministic marker contracts.
5. Run targeted verification (shell contract tests, format/clippy gates), then open and merge PR.

## Affected Modules

- `scripts/deploy/validate_compose_topology_contract_lane.sh`
- `scripts/deploy/check_compose_topology_contract_policy.sh`
- `scripts/deploy/validate_deployment_assets_live.sh`
- `scripts/deploy/test_validate_compose_topology_contract_lane.sh`
- `scripts/deploy/test_check_compose_topology_contract_policy.sh`
- `docs/ops/deployment.md`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: Existing compose-topology pass path regressions.
  - Mitigation: Keep existing markers intact and add additive taxonomy markers; verify legacy checks.
- Risk: Over-broad reason-code surface causes unstable outputs.
  - Mitigation: Use fixed ordered reason-code CSV and deterministic field naming.
- Risk: CI/runtime budget drift from expanded tests.
  - Mitigation: Keep tests scoped to existing scripts and dry-run/tamper fixtures.

## Interfaces / Contracts

- New lane marker surfaces:
  - `packaging_reason_taxonomy_version=<version>`
  - `packaging_reason_codes_csv=<csv>`
  - `packaging_contract_evidence_status=verified`
- Policy checker deterministic failure reasons:
  - `compose_topology_policy_packaging_reason_taxonomy_version_mismatch`
  - `compose_topology_policy_packaging_reason_codes_csv_mismatch`
  - `compose_topology_policy_packaging_contract_evidence_status_mismatch`

## ADR

No ADR required (no dependency additions, no wire/protocol migration).
