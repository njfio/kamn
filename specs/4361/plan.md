# Plan: #4361 Key-Source Reason Mapping

## Approach

1. Add key-source taxonomy constants in runtime integration policy checker.
2. Add strict checks for key-source contract-version fields and real-node command marker.
3. Derive observed key-source reason subset from full `reason_codes` and emit deterministic value.
4. Update docs with exact taxonomy markers consumed by release/runtime evidence checks.
5. Verify with targeted script tests and docs contract tests.

## Affected Files

- `scripts/kolme/check_local_kamn_live_runtime_integration_policy.py`
- `docs/security/key-management.md`
- `docs/foundation/release-gonogo-checklist.md`
- tests that assert policy output/doc markers.
