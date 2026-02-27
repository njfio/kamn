# Plan: Issue #6039

## Approach
1. Add deterministic scenario fixtures for critical and non-critical required scenarios.
2. Write RED test for reason-code precedence contract.
3. Add completeness/Go path tests and verify aggregate counters and ID lists.
4. Keep production behavior unchanged unless tests expose mismatch.

## Affected Modules
- `crates/kamn-core/src/data_layer_m11_hardening_readiness.rs`

## Risks / Mitigations
- Risk: assertions may encode incidental ordering beyond contract.
  Mitigation: assert explicit reason precedence and deterministic sorted IDs exposed by BTreeMap iteration.

## Interfaces / Contracts
- No public API changes.
- Test-only contract coverage additions.
