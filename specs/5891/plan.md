# Plan: Issue #5891 - Expand Default Panic-Path Audit Coverage to kamn-agent-lib

## Approach
1. Update `DEFAULT_RUNTIME_ROOTS` in `scripts/ci/check_no_production_expect.py` to include `crates/kamn-agent-lib/src`.
2. Run default checker, checker regression suite, and scoped agent-lib checker.
3. Publish RED/GREEN (if any) and conformance evidence in PR.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`

## Risks and Mitigations
- Risk: newly covered root surfaces violations and fails default gate.
  - Mitigation: run scoped checker first and preserve deterministic outputs.

## Interfaces / Contracts
- No schema or taxonomy changes.
- Existing output marker contract remains unchanged.

## ADR
- Not required.
