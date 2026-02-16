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

## Remediation

- Replace panic-style control flow with typed error propagation and deterministic error markers.
- Replace unsafe fallback defaults with explicit fail-closed validation and actionable diagnostics.
