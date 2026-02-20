# Issue #4006 Plan

## Approach
1. Add `scripts/ci/check_capacity_ci_dry_run_governance.py` to validate:
   - performance smoke report contract markers,
   - runtime go/no-go dry-run report markers,
   - `scripts/ci/test_ci_tools.sh` fast-mode required/forbidden entries,
   - `.github/workflows/ci-fast-gate.yml` forbidden run-mode entry,
   - `docs/ci/strategy.md` contract + remediation markers.
2. Add threshold/taxonomy fixture at `fixtures/ci/capacity_ci_dry_run_governance_thresholds.env`.
3. Add contract suite `crates/kamn-core/tests/capacity_ci_dry_run_governance_contract.rs` with Unit/Functional/Integration/Regression/Performance cases.
4. Wire fast-mode CI tool execution by adding the new contract test command to `scripts/ci/test_ci_tools.sh`.
5. Add docs section/markers for governance checker and remediation map in `docs/ci/strategy.md`.

## Affected Modules
- `scripts/ci/check_capacity_ci_dry_run_governance.py` (new)
- `fixtures/ci/capacity_ci_dry_run_governance_thresholds.env` (new)
- `crates/kamn-core/tests/capacity_ci_dry_run_governance_contract.rs` (new)
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `specs/4006/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: reason-code drift or unstable ordering causes flaky governance outputs.
  - Mitigation: threshold fixture defines ordered reason CSV; checker normalizes to ordered list.
- Risk: integration contract overfits to volatile report fields.
  - Mitigation: validate stable schema/required markers only; avoid non-contract incidental fields.
- Risk: shell-surface growth.
  - Mitigation: implement logic in Python + Rust test; shell changes limited to one CI entrypoint line.

## Interfaces and Contracts
- No new dependencies.
- No workflow command behavior changes; only contract coverage enforcement in existing fast-mode test set.
- New checker output schema:
  - `kamn.ci.capacity-ci-dry-run-governance-report.v1`
