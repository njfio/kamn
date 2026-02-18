# Declarative Policy Migration Telemetry (Issue #4881)

## Scope

This telemetry snapshot tracks the first large migration wave of policy-checker wrappers
delegated through `scripts/framework/declarative_policy_checker.py` via the exec-dispatch registry.

## Wave Selection Contract

Migrated wrapper candidates in this wave satisfy all of the following:

- Wrapper path is under `scripts/` and matches `*check_*.sh` or `*validate_*.sh`
- Registry interpreter is `python3`
- Registry target ends with `_contract.py` or `_policy_contract.py`
- Target source line count is `<= 1500`

## Measured Totals

- Total python contract-target wrappers in registry: `154`
- Selected/migrated in this wave: `100`
- Residual check/validate wrappers above threshold (`>1500` lines): `3`
- Remaining non-check/validate contract-target wrappers: `51`

## Residual Backlog

High-size check/validate wrappers not included in this wave:

- `scripts/deploy/check_gonogo_evidence_policy.sh` -> `scripts/deploy/gonogo_evidence_contract.py` (`1916` lines)
- `scripts/runtime/check_local_full_stack_integration_live_policy.sh` -> `scripts/runtime/local_full_stack_integration_live_contract.py` (`1929` lines)
- `scripts/runtime/validate_local_full_stack_integration_live.sh` -> `scripts/runtime/local_full_stack_integration_live_contract.py` (`1929` lines)

These remain explicit backlog candidates for decomposition before declarative migration expansion.
