# Plan: Issue #4451

Status: Completed
Issue: #4451

## Approach

1. Add RED coverage in runtime policy and contract-lane shell tests for:
   - deterministic normalized reason value marker (`reason_codes_value`)
   - deterministic parity evidence-output normalization marker
   - explicit mapping of extraction evidence drift to taxonomy reason.
2. Implement normalized reason mapper and parity evidence-output marker emission in
   `scripts/runtime/local_full_stack_integration_live_contract.py`.
3. Propagate normalized markers through
   `scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh`.
4. Update `docs/architecture/runtime.md` and docs contract tests to pin the new markers.
5. Run targeted RED/GREEN loops, then scoped verification (`fmt`, `clippy`, targeted tests).

## Affected Modules

- `scripts/runtime/local_full_stack_integration_live_contract.py`
- `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
- `scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh`
- `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
- `docs/architecture/runtime.md`
- `crates/kamn-core/tests/runtime_architecture_docs.rs`
- `specs/4451/*`

## Risks and Mitigations

- Risk: existing policy tests rely on raw fail-check outputs.
  - Mitigation: preserve `failed_checks` and failure stderr behavior while adding normalized markers.
- Risk: marker expansion causes brittle string checks.
  - Mitigation: use explicit constant markers and assert only stable contract strings.

## Interfaces / Contracts

- Local full-stack integration policy report contract:
  - Add deterministic `reason_codes_value` normalization field.
  - Add deterministic parity evidence-output normalization field.
- Contract-lane output/report contract:
  - Surface normalized reason and parity evidence-output markers from summary/policy.
- Runtime architecture docs contract:
  - Include new deterministic taxonomy/normalization references.

## ADR

Not required: no new dependency or architecture decision, only deterministic policy
mapping/output normalization and associated contract tests/docs.
