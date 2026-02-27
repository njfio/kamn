# Plan: Issue #6139

## Approach
1. Reproduce the current gate behavior and validate issue context on current `main`.
2. Replace filesystem recursive traversal with git-tracked file enumeration.
3. Add regression test that injects an untracked shell test file and asserts no count/ratio change.
4. Run scoped tests (`shell_test_surface_ratio_policy`) and quality checks (`fmt`, `clippy`).
5. Publish PR evidence with AC-to-test mapping and shell-surface impact markers.

## Affected Modules
- `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs`
- `specs/6139/spec.md`
- `specs/6139/plan.md`
- `specs/6139/tasks.md`

## Risks / Mitigations
- Risk: `git` invocation could fail in unusual environments.
  Mitigation: fail closed with existing deterministic reason code (`threshold_value_invalid`) and explicit detail.
- Risk: contract drift in existing reason taxonomy/report schema.
  Mitigation: preserve existing reason constants and fixture marker structure.

## Interfaces / Contracts
- No public runtime API changes.
- Existing test policy schema markers remain unchanged.
