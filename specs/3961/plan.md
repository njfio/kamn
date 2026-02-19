# Issue #3961 Plan

- Issue: #3961
- Status: Completed
- Spec: `specs/3961/spec.md`

## Implementation Approach
1. Add a Rust docs-contract/parity test targeting existing deployment preflight lane script + docs markers.
2. RED: assert closure markers in production next-steps doc that are not yet declared.
3. GREEN: add missing closure marker section/guard command in next-steps doc and ensure marker parity with lane scripts/docs.
4. Run targeted contract tests.

## Affected Modules
- `crates/kamn-core/tests/deployment_hardening_lane_contract.rs`
- `docs/plans/2026-02-14-production-service-next-steps.md`

## Risks and Mitigations
- Risk: brittle prose checks.
  - Mitigation: pin deterministic marker keys/commands/reason codes only.
- Risk: duplicated marker taxonomy diverges.
  - Mitigation: parity test validates docs marker set against lane implementation references.

## Contracts and Interfaces
- Required markers include:
  - `deployment_preflight_passed`
  - `dry_run_no_commands_executed`
  - `preflight_budget_exceeded`
  - `signer_provenance_present`
  - `signer_provenance_sha256_valid`
  - `signer_rotation_rehearsal_drift_detected`
  - `signer_rotation_promotion_stalled`

## Verification Strategy
- RED: run new test before adding next-steps closure markers.
- GREEN: add closure marker section and rerun.
- REGRESSION: rerun deployment hardening contract test.
