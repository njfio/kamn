# 7032 Restore Fast Gate cargo-audit advisory policy

## Objective

Restore the PR quality gate after Fast Gate failed before the repository-owned
cargo-audit policy checker could emit deterministic security evidence.

## Inputs/Outputs

- Input: `cargo-audit-report.json` generated from the current workspace lockfile.
- Input: `.ci/cargo-audit-waivers.json` with explicit package-scoped waivers.
- Output: `ci-cargo-audit-policy.json` with deterministic pass/fail markers.
- Output: Fast Gate and Deep Validate cargo-audit steps that preserve audit JSON
  and still fail closed through the policy checker.

## Boundaries/Non-goals

- Do not weaken cargo-audit, lint, formatting, clippy, proof, or governance gates.
- Do not broadly modernize dependencies beyond the reported advisories.
- Do not replace the existing cargo-audit policy framework.
- Do not start MVP feature expansion until this quality-gate blocker is repaired.
- Do not silently pass unmaintained, unsound, yanked, unknown-severity, or
  unwaived advisories.

## Failure modes

- `cargo audit --json` exits nonzero before policy evidence is emitted.
- Patchable vulnerable packages remain pinned to affected versions.
- Warning advisories are not represented in policy output.
- Unknown-severity vulnerabilities pass implicitly.
- Waivers omit package scope, tracking issue, expiry, or deterministic reason.
- CI uploads no cargo-audit artifacts when the gate fails.

## Acceptance criteria

- [ ] Patchable Cargo.lock advisories are updated or removed without weakening
  dependency/security checks.
- [ ] Fast Gate and Deep Validate still run cargo-audit and preserve
  `cargo-audit-report.json` for policy evaluation.
- [ ] The repo-owned policy checker runs after cargo-audit and fails closed on
  unapproved vulnerabilities, unknown severities, invalid waivers, expired
  waivers, and unapproved warning advisories.
- [ ] Any remaining upstream-blocked advisory is represented by an explicit
  package-scoped waiver with tracking issue and expiry.
- [ ] Targeted policy tests prove the previous premature-exit path red first
  and the repaired path green.
- [ ] `cargo fmt --check`, strict workspace clippy, `make check`, governance
  ratio, and shell-surface ratio gates pass.

## Files to touch

- `Cargo.lock`
- `crates/kamn-core/Cargo.toml` if removing an unused vulnerable feature is
  needed and test evidence proves the runtime contract remains intact.
- `.ci/cargo-audit-waivers.json`
- `.github/workflows/ci-fast-gate.yml`
- `.github/workflows/ci-deep-validate.yml`
- `scripts/ci/check_cargo_audit_policy.py`
- `scripts/ci/test_check_cargo_audit_policy.sh`
- `crates/kamn-core/tests/ci_fast_gate_workspace_premerge_contract.rs`
- `specs/7032-restore-fast-gate-cargo-audit-advisory-policy.md`

## Error semantics

- Missing or invalid cargo-audit report: fail closed with existing report reason.
- Unknown advisory severity: fail closed unless explicitly waiver-tracked.
- Unapproved vulnerability above threshold: fail closed.
- Unapproved warning advisory: fail closed.
- Expired or malformed waiver: fail closed.
- Workflow cargo-audit invocation may capture nonzero cargo-audit exit status,
  but the final gate result must come from the policy checker and must remain
  nonzero when policy rejects the report.

## Test plan

- Red: reproduce CI failure from job `83790804681` where cargo-audit exits before
  `ci-cargo-audit-policy.json` can be produced.
- Red: add policy fixture coverage for warning advisories and prove the existing
  checker does not fail closed on them.
- Green: run targeted compatible updates for patchable packages:
  `rustls-webpki` to `0.103.13`, `quinn-proto` to `0.11.15`,
  `rand@0.8.5` to `0.8.6`, and `rand@0.9.2` to `0.9.3` where cargo permits.
- Green: remove unused vulnerable feature dependencies only with targeted compile
  and runtime contract evidence.
- Green: run `bash scripts/ci/test_check_cargo_audit_policy.sh`.
- Green: run local cargo-audit JSON generation followed by
  `python3 scripts/ci/check_cargo_audit_policy.py --audit-json ...`.
- Green: run `cargo fmt --check`.
- Green: run
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Green: run `make check`.
- Green: run governance feature-commit ratio and shell-surface ratio checks.

## Observed red evidence

- Fast Gate job `83790804681` failed on
  `cargo audit --json > cargo-audit-report.json` at PR head `dc8e45b9`.
- Downloaded report contained seven vulnerability rows plus warning rows for
  unmaintained, unsound, and yanked packages.
- Dry-run update probes showed compatible updates are available for
  `rustls-webpki`, `quinn-proto`, and both `rand` major lines.

## Implementation notes

- Cargo-audit nonzero exit status is captured by Fast Gate and Deep Validate so
  `cargo-audit-report.json` remains available to the repo-owned policy checker.
- The policy checker now evaluates cargo-audit `warnings` rows as first-class
  policy inputs and fails closed on unwaived warning advisories.
- Unknown-severity vulnerabilities remain fail-closed unless an explicit
  package-scoped waiver covers the advisory, package, tracking issue, and
  expiry.
- `libp2p` no longer enables the unused DNS feature from KAMN's direct
  dependency declaration. The lockfile can still contain optional upstream DNS
  packages, so remaining cargo-audit rows are tracked with explicit waivers
  rather than represented as eliminated.
- The native libp2p adapter runtime test remains the integration evidence that
  removing the direct DNS feature did not disconnect KAMN's current TCP/noise/
  yamux transport path.
