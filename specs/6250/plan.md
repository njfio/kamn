# Issue 6250 Plan

## Approach
1. Capture baseline shell/rust ratio using the existing guardrail checker.
2. Migrate the shell-rust guardrail regression lane from shell test wrapper execution to a Rust integration test:
   - add `crates/kamn-core/tests/ci_shell_rust_ratio_guardrail_contract.rs`
   - wire fast-mode CI tools selector to call the Rust test
   - remove retired shell test wrapper script.
3. Update command-surface contract expectations and CI strategy docs for the new Rust lane.
4. Re-measure ratio and record before/after evidence in `docs/planning/r59-followup.md`.

## Affected Modules
- `crates/kamn-core/tests/ci_shell_rust_ratio_guardrail_contract.rs` (new)
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `scripts/ci/test_check_shell_rust_ratio_guardrail.sh` (removed)
- `docs/ci/strategy.md`
- `docs/planning/r59-followup.md` (new)
- `specs/6250/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: command-surface contract drift after selector changes.
  - Mitigation: update and run `scripts/ci/test_ci_tools_command_surface_contract.sh`.
- Risk: migration changes checker behavior unintentionally.
  - Mitigation: Rust contract test executes the actual wrapper script with pass/warn/fail/error threshold fixtures and validates reason markers.
- Risk: ratio evidence is stale or inferred.
  - Mitigation: compute metrics directly with `check_shell_rust_ratio_guardrail.sh` before and after changes.

## Interfaces
- No production API changes.
- CI interface retained:
  - checker invocation remains `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...`
  - fast-mode CI selector now validates this lane through a Rust integration test command.
