# ADR: Cargo-Audit Policy Gate in Required CI

## Context

The repository has deterministic dependency-advisory fixture tests, but no required CI gate that executes a live Rust dependency vulnerability scan and blocks unwaived high-severity findings. This leaves a production-readiness gap for supply-chain risk enforcement.

Issue: `#5941`

## Decision

Add a fail-closed cargo-audit policy gate to required CI and deep validation workflows:

- Generate live advisory report:
  - `cargo audit --json > cargo-audit-report.json`
- Enforce deterministic policy:
  - `python3 scripts/ci/check_cargo_audit_policy.py --audit-json cargo-audit-report.json --waiver-file .ci/cargo-audit-waivers.json --threshold-max-severity moderate --output-json ci-cargo-audit-policy.json`
- Archive report and policy outputs as workflow artifacts.

Waiver policy is explicit and tracked:

- Waiver file schema: `kamn.ci.cargo-audit-waiver.v1`
- Required waiver fields: `advisory_id`, `reason`, `tracking_issue` (`#<issue-id>`), `expires_on`
- Expired or malformed waivers fail closed.

## Consequences

Positive:

- Required CI now blocks unwaived high/critical dependency advisories.
- Waiver exceptions become auditable and time-bounded.
- Security evidence is archived per run.

Trade-offs:

- CI runtime increases due to cargo-audit installation/execution.
- Waiver governance adds process overhead for temporary exceptions.

Operational follow-up:

- Keep `.ci/cargo-audit-waivers.json` empty by default and retire waivers quickly.
- Refresh policy/doc markers if cargo-audit JSON schema evolves.
