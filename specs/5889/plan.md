# Plan: Issue #5889 - sdk_direct Unsafe Env Fallback Regression Remediation

## Approach
1. Reproduce RED state with `scripts/ci/check_no_production_expect.sh` and capture failing evidence.
2. In `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`, replace remaining direct fallback-default usages with helper-based resolution (`env_var_or_default` / `env_var_or_else`) while preserving defaults.
3. Re-run checker lanes and e2e-harness tests to confirm GREEN and no behavior drift.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`

## Risks and Mitigations
- Risk: subtle default-value drift in live probes.
  - Mitigation: preserve exact env keys/default constants and validate with harness tests.
- Risk: checker-specific pattern regressions reintroduced by formatting or refactor.
  - Mitigation: run default + scoped checker commands and include regex conformance evidence in PR.

## Interfaces / Contracts
- No public API changes.
- Panic-path replacement contract remains `scripts/ci/check_no_production_expect.py` output markers and status semantics.

## ADR
- Not required (no dependency/protocol/architecture change).
