# Tasks — Issue #4833

- [x] T1 (Red): add failing shell-surface intake docs-contract test before docs/template updates.
  Evidence:
  - `bash scripts/ci/test_shell_surface_issue_intake_contract.sh` failed with:
    `expected AGENTS shell-surface DoR gate marker '## Shell-Surface DoR Gate' in .../AGENTS.md`
- [x] T2 (Green): add shell-surface DoR/DoD marker contracts to AGENTS/CONTRIBUTING and issue templates.
  Evidence:
  - Added marker blocks and keys in `AGENTS.md` and `.github/CONTRIBUTING.md`.
  - Added shell-surface estimate fields to all issue templates.
- [x] T3 (Refactor): add deterministic docs-contract test and wire into CI tools regression.
  Evidence:
  - Added `scripts/ci/test_shell_surface_issue_intake_contract.sh`.
  - `scripts/ci/test_ci_tools.sh` now runs the new contract test in fast/full blocks.
- [x] T4 (Verify): run deterministic suites and regression coverage.
  Evidence:
  - `bash scripts/ci/test_shell_surface_issue_intake_contract.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
