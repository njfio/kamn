# Issue #4957 Plan

- Issue: #4957
- Status: Implemented

## Approach
- Deliver story through tasks:
  - `#4964` hard ceiling checker lifecycle completion.
  - `#4965` fast-gate wiring for ceiling+ratio required checks.
  - `#4966` ratchet/waiver governance enforcement.
- Synchronize story lifecycle docs post task-closure.

## Affected Modules
- `scripts/ci/check_shell_loc_hard_ceiling.sh`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/check_shell_surface_threshold_ratchet.py`
- `scripts/ci/test_check_shell_loc_hard_ceiling.sh`
- `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `.github/workflows/ci-fast-gate.yml`

## Risks and Mitigations
- Risk: CI wiring or waiver policy drift reduces merge protection.
- Mitigation: deterministic contract tests and required fast-gate checks.

## Interface Contract
- Preserve checker reason-taxonomy/report schemas used by CI telemetry/policy lanes.

## ADR
- Not required.
