# Plan: Issue #4456

Status: Completed
Issue: #4456

## Approach

1. Add red-path fixture assertions to dependency posture checker tests for docs mismatch drift.
2. Add red-path fixture assertions to workspace license policy tests for malformed and missing
   metadata structures.
3. Extend release go/no-go checklist docs and docs-contract assertions for dependency/license
   mismatch acceptance and regression markers.
4. Run scoped red/green verification and repository hygiene commands.

## Affected Modules

- `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- `scripts/ci/test_check_workspace_license_policy.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `specs/4456/*`

## Risks and Mitigations

- Risk: new assertions become brittle against unrelated docs formatting edits.
  - Mitigation: assert only stable gate markers and reason tokens.
- Risk: shell test fixture edits become noisy.
  - Mitigation: keep fixture mutations local and deterministic via temporary files.

## Interfaces / Contracts

- Dependency posture gate continues to emit deterministic reason markers for docs drift:
  - `readme_*_reference_missing`
  - `readme_no_default_features_marker_missing`
  - `ci_strategy_*_missing`
- Workspace license policy checker failure reasons remain explicit:
  - `manifest_not_found`
  - `manifest_invalid_toml`
  - `package_section_missing`
  - `license_missing`
  - `license_mismatch`

## ADR

Not required: no architecture or dependency changes.
