## Objective

Add earlier fail-closed `cargo-audit` feedback to `Fast Gate (PR)` so dependency vulnerability
regressions surface before the slower `Workspace Pre-Merge Gate (PR)` lane completes.

## Inputs/Outputs

- Inputs:
  - `.github/workflows/ci-fast-gate.yml`
  - existing `cargo-audit` commands and waiver policy
  - existing workflow/docs contract test coverage
- Outputs:
  - `Fast Gate (PR)` executes `cargo-audit` under the Rust-scope selector
  - existing waiver/policy enforcement and security artifact upload are available in fast gate
  - workflow/docs contract tests pin the new fast-gate security surface

## Boundaries/Non-goals

- No waiver schema changes
- No `cargo-audit` threshold or policy logic changes
- No parser/runtime/product behavior changes
- No broader CI job decomposition in this slice

## Failure modes

- `Fast Gate (PR)` omits the `cargo-audit` scan entirely
- `Fast Gate (PR)` runs `cargo-audit` without the waiver/policy checker
- Security artifact upload is missing from the fast gate path
- Docs/workflow contracts drift and silently remove the fast-gate cargo-audit path

## Acceptance criteria

- [ ] `Fast Gate (PR)` installs `cargo-audit` under the existing Rust-scope selector
- [ ] `Fast Gate (PR)` runs `cargo audit --json > cargo-audit-report.json`
- [ ] `Fast Gate (PR)` enforces the existing checker against `.ci/cargo-audit-waivers.json`
- [ ] `Fast Gate (PR)` uploads `cargo-audit-report.json` and `ci-cargo-audit-policy.json` artifacts
- [ ] CI docs/contracts record that cargo-audit feedback exists in `Fast Gate (PR)`
- [ ] `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract -- --nocapture` passes

## Files to touch

- `specs/6491-move-cargo-audit-into-fast-gate.md`
- `.github/workflows/ci-fast-gate.yml`
- `docs/ci/strategy.md`
- `docs/architecture/adr-cargo-audit-ci-gate.md`
- `crates/kamn-core/tests/ci_fast_gate_workspace_premerge_contract.rs`

## Error semantics

- `cargo-audit` findings continue to fail closed through
  `scripts/ci/check_cargo_audit_policy.py`
- Waiver handling remains unchanged and still fails closed on malformed or expired entries
- No new runtime error semantics are introduced outside CI

## Test plan

- Extend the existing workflow/docs contract test to require fast-gate cargo-audit markers
- Run:
  - `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract -- --nocapture`

## Phase 6 integration plan

- Verify the integrated path through the real `ci-fast-gate` workflow file and strategy/ADR docs
- Open a PR and rely on actual `Fast Gate (PR)` CI execution as final integration evidence

## Deviations

- None
