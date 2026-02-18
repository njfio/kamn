# Tasks — Issue #4887

- [x] T1 (Red): add failing spec-derived checks before implementation.
  - `bash scripts/ci/test_check_pr_ci_declaration.sh` failed with `Expected failure when shell_loc_delta_actual is non-numeric`.
  - `cargo test -p kamn-core --test shell_surface_governance_docs` failed for missing mitigation-link guidance markers in PR template and CI strategy docs.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Added numeric validation and mitigation-link enforcement in `scripts/ci/check_pr_ci_declaration.sh`.
  - Added mitigation-link guidance markers in `.github/pull_request_template.md` and `docs/ci/strategy.md`.
  - Added docs-contract coverage in `crates/kamn-core/tests/shell_surface_governance_docs.rs`.
- [x] T3 (Refactor): strengthen deterministic docs/template contract tests.
  - Extended `scripts/ci/test_pr_template_shell_surface_markers_contract.sh` to validate CI strategy guidance markers and checker linkage semantics.
  - Extended `scripts/ci/test_check_pr_ci_declaration.sh` with mitigation-link and numeric-field validation cases.
- [x] T4 (Verify): run required test tiers and capture evidence.
  - `cargo fmt --check`
  - `cargo test -p kamn-core --test shell_surface_governance_docs`
  - `bash scripts/ci/test_check_pr_ci_declaration.sh`
  - `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh`
