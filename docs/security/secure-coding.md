# Secure Coding

## Panic Reachability Policy

- panic_path_reachability_policy=fail_closed
- unsafe_fallback_default_policy=fail_closed
- production panic-path checker command:
  - `scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json`
- production panic-path checker command (multi-root):
  - `scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --root crates/kamn-core/src --output-json /tmp/no-production-expect-report.json`

## Failure Cases

- production_panic_path_violation_markers=.expect(,panic!,unreachable!,unsafe_env_fallback_default
- production_panic_path_violation_class=panic_reachability|unsafe_fallback
- panic_reachability:
  - `.expect(` in production paths before `#[cfg(test)]`
  - `panic!` in production paths before `#[cfg(test)]`
  - `unreachable!` in production paths before `#[cfg(test)]`
- unsafe_fallback:
  - environment-derived secrets or signer inputs that use inline default fallbacks such as
    `.unwrap_or(...)` or `.unwrap_or_else(...)` on `std::env::var(...)`.

## Deterministic Taxonomy and Runtime Evidence

- panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1
- panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default
- panic_replacement_reason_codes_value=none|<csv>
- panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration
- runtime_panic_replacement_evidence_status=verified|violation
- runtime_panic_replacement_evidence_violation_count=<n>
- runtime_panic_replacement_evidence_files_csv=none|<csv>
- runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv

## Remediation

- Replace panic-style control flow with typed error propagation and deterministic error markers.
- Replace unsafe fallback defaults with explicit fail-closed validation and actionable diagnostics.
