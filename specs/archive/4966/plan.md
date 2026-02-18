# Issue #4966 Plan

- Issue: #4966
- Status: Implemented

## Approach
- Implement shell-surface threshold-ratchet checker with deterministic waiver validation.
- Wire checker into fast-gate CI and contract suites.
- Validate reason taxonomy + report schema via regression tests.

## Affected Modules
- `.ci/shell-surface-threshold-ratchet-exception.json`
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/check_shell_surface_threshold_ratchet.py`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh`
- `scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`

## Risks and Mitigations
- Risk: waiver metadata bypass could allow ratio regression without mitigation linkage.
- Mitigation: deterministic checker validation and CI merge-blocking wiring.

## Interface Contract
- Preserve reason taxonomy/report schema consumed by fast-gate telemetry and policy checks.

## ADR
- Not required.
