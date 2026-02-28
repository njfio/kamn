# Issue 6266 Plan

## Approach
1. Add Python cache ignore markers to `.gitignore`.
2. Add a contract test that reads `.gitignore` and asserts required markers.
3. Run conformance commands and targeted test.
4. Verify `git status --short` no longer reports `scripts/framework/__pycache__/`.

## Affected paths
- `.gitignore`
- `crates/kamn-core/tests/workspace_gitignore_python_cache_policy_contract.rs`
- `specs/6266/spec.md`
- `specs/6266/plan.md`
- `specs/6266/tasks.md`

## Risks and mitigations
- Risk: marker mismatch between spec and test.
  - Mitigation: derive test assertions directly from spec markers.
- Risk: narrow marker does not cover bytecode variants.
  - Mitigation: include `*.pyc`, `*.pyo`, and `*.pyd`.

## Contract notes
- This is repository hygiene policy only.
- No runtime logic or API contracts are changed.
