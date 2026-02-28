# Issue 6259 Plan

## Approach
1. Add red tests for typed reason enum conversion and invalid parse behavior.
2. Introduce `DataLayerShellNeutralPolicyReasonCode` enum with conversion helpers.
3. Update `DataLayerShellNeutralPolicyReport.reason_codes` to typed enum vector.
4. Update tests/callsites and root exports.
5. Run targeted tests for shell-neutral policy contracts.

## Affected paths
- `crates/kamn-core/src/data_layer_shell_neutral_policy.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_shell_neutral_policy.rs`
- `specs/6259/spec.md`
- `specs/6259/plan.md`
- `specs/6259/tasks.md`

## Risks and mitigations
- Risk: boundary string drift.
  - Mitigation: explicit `as_str` mapping tests for every enum variant.
- Risk: downstream compile break from type change.
  - Mitigation: update in-workspace tests/callsites in same change.

## Contract notes
- Migration preserves canonical reason string vocabulary.
- Scope is intentionally bounded to a single reason-code domain.
