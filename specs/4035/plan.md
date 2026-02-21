# Issue #4035 Plan - Regression + Remediation-Marker Coverage for License Metadata Mismatches

## Approach
1. Add RED docs-contract tests that require dependency-license remediation markers in both strategy and ops docs for each checker reason code.
2. Add workspace checker regression coverage for deterministic multi-reason mismatch behavior.
3. Add dependency-license remediation marker blocks to `docs/ci/strategy.md` and `docs/ops/configuration.md`.
4. Re-run targeted tests plus formatting/lint gates and capture RED/GREEN evidence.

## Affected Modules
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `crates/kamn-core/tests/workspace_license_policy_contract.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/4035/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: reason-code list drift between checker and docs.
  - Mitigation: docs-contract loops derive required remediation marker keys from a single reason-code CSV constant.
- Risk: strategy/ops wording divergence.
  - Mitigation: enforce same key namespace and per-reason marker presence in both docs.
- Risk: overly broad docs edits causing unrelated failures.
  - Mitigation: isolate edits to dependency-license governance section only.

## Interfaces / Contracts
- Checker taxonomy version:
  - `kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1`
- Reason code contract:
  - `expected_license_empty`
  - `no_crate_manifests_found`
  - `license_policy_file_not_found`
  - `license_policy_marker_mismatch`
  - `manifest_not_found`
  - `manifest_invalid_toml`
  - `package_section_missing`
  - `license_missing`
  - `license_mismatch`
  - `metadata_governance_local_heavy_opt_in_required`
- Remediation marker namespace:
  - `metadata_governance_remediation_map_version=v1`
  - `metadata_governance_remediation.<reason_code>=...`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_dependency_license_metadata_governance_remediation_markers -- --exact`
- GREEN:
  - rerun RED tests after docs updates
  - `cargo test -p kamn-core --test workspace_license_policy_contract regression_workspace_license_policy_checker_reports_deterministic_multi_reason_mismatch_markers -- --exact`
- VERIFY:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
