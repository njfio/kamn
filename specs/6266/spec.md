# Issue 6266 Spec

Status: Reviewed
Priority: P2
Milestone: R59 Swarm Gap Closure

## Objective
Stop Python bytecode/cache artifacts from surfacing as untracked workspace noise
by enforcing repository-level ignore rules with regression coverage.

## Inputs/Outputs
Inputs:
- Current repository ignore policy in `.gitignore`.
- Existing local artifact path `scripts/framework/__pycache__/`.

Outputs:
- Updated `.gitignore` rules for Python cache artifacts.
- Regression test that fails if required ignore markers are removed.

## Boundaries/Non-goals
In scope:
- Add explicit ignore markers for `__pycache__/` and Python bytecode files.
- Validate that `git status --short` does not show tracked Python cache paths.
- Add contract-style regression test for ignore markers.

Out of scope:
- Deleting local cache files from user machines.
- Expanding ignore policy to unrelated generated artifacts.
- Runtime or product behavior changes.

## Failure modes
- FM-1: Missing ignore marker causes cache directories to appear in `git status`.
- FM-2: Ignore marker drift/regression reintroduces repository noise.

## Acceptance criteria (testable booleans)
- AC-1: `.gitignore` contains repository-level Python cache ignore markers
  (`__pycache__/` and bytecode variants).
- AC-2: `git status --short` does not report `scripts/framework/__pycache__/`.
- AC-3: A regression test fails if required `.gitignore` markers are removed.

## Files to touch
- `.gitignore`
- `crates/kamn-core/tests/workspace_gitignore_python_cache_policy_contract.rs`
- `specs/6266/spec.md`
- `specs/6266/plan.md`
- `specs/6266/tasks.md`

## Error semantics
- Fail closed in regression test with explicit assertion messages.
- No silent fallbacks in policy verification.

## Test plan
Conformance:
- C-01 (AC-1): `rg -n '^(__pycache__/|\*\.pyc|\*\.pyo|\*\.pyd)$' .gitignore`
  returns expected markers.
- C-02 (AC-2): `git status --short` does not include
  `scripts/framework/__pycache__/`.
- C-03 (AC-3):
  `cargo test -p kamn-core --test workspace_gitignore_python_cache_policy_contract`
  passes.

Regression:
- `cargo test -p kamn-core --test workspace_gitignore_python_cache_policy_contract -- --nocapture`
