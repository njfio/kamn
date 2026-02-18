# Plan — Issue #4834

## Approach

1. Add a docs-contract test that expects shell-surface declaration markers in PR template and declaration checker script.
2. Update `.github/pull_request_template.md` with required shell-surface declaration checkboxes and marker fields.
3. Extend `scripts/ci/check_pr_ci_declaration.sh` to enforce shell-sensitive declarations for shell-sensitive file changes.
4. Extend `scripts/ci/test_check_pr_ci_declaration.sh` with shell-sensitive pass/fail cases.
5. Add the new docs-contract test to `scripts/ci/test_ci_tools.sh` and validate fast-mode regression.
6. Update CI strategy documentation for PR shell-surface declaration governance.

## Affected Modules

- `.github/pull_request_template.md`
- `scripts/ci/check_pr_ci_declaration.sh`
- `scripts/ci/test_check_pr_ci_declaration.sh`
- `scripts/ci/test_pr_template_shell_surface_markers_contract.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: false negatives in declaration checker for shell-sensitive changes.
  Mitigation: explicit shell-sensitive path matching plus force-sensitivity test env coverage.
- Risk: contributor friction from stricter PR template requirements.
  Mitigation: deterministic marker fields and clearly documented accepted value set.
- Risk: drift between PR template and checker.
  Mitigation: dedicated docs-contract test enforces both surfaces.

## Interfaces / Contracts

- PR template shell declaration markers:
  - `shell_loc_delta_actual:`
  - `rust_loc_delta_actual:`
  - `shell_to_rust_ratio_delta_actual:`
  - `shell_surface_ratio_target_status: improved|neutral|regressed_with_waiver`
  - `shell_surface_mitigation_issue:`
- Checker shell-sensitive enforcement knobs:
  - `SHELL_SURFACE_DECLARATION_FORCE_SENSITIVE=true|false|auto`
- Checker accepts ratio target statuses:
  - `improved`
  - `neutral`
  - `regressed_with_waiver`

## ADR

No ADR required. No dependency/protocol boundary changes were introduced.
