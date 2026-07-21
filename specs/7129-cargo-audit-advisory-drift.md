# Issue 7129: Restore Cargo Audit Policy

## Objective

Restore the fail-closed cargo-audit policy after two new RustSec advisories by
updating only the affected lockfile entries to their first patched versions.

## Inputs/Outputs

- Input: `Cargo.lock` entries for `anyhow` and `crossbeam-epoch`.
- Input: RustSec advisories `RUSTSEC-2026-0190` and `RUSTSEC-2026-0204`.
- Output: `anyhow` locked at `1.0.103` or newer.
- Output: `crossbeam-epoch` locked at `0.9.20` or newer.
- Output: an audit policy report with no violation from either advisory.

## Boundaries/Non-goals

- Change no manifests, source code, audit threshold, or waiver policy.
- Do not add, remove, or broadly upgrade dependencies.
- Do not modify issue #7127 format behavior.
- Preserve reproducibility under `--locked` commands.

## Failure Modes

- Either vulnerable lockfile version remains.
- Cargo resolves unrelated package updates.
- The lockfile becomes inconsistent with workspace manifests.
- Cargo audit still reports either advisory as a policy violation.
- A waiver hides an advisory instead of applying the available patch.

## Acceptance Criteria

- [ ] `anyhow` resolves to at least `1.0.103`.
- [ ] `crossbeam-epoch` resolves to at least `0.9.20`.
- [ ] Only those two package versions and checksums change in `Cargo.lock`.
- [ ] No cargo-audit threshold or waiver changes are made.
- [ ] `cargo metadata --locked` succeeds.
- [ ] The cargo-audit policy checker passes.
- [ ] The manually dispatched `ci-fast-gate` reaches downstream Rust gates.

## Files to Touch

- `Cargo.lock`
- `specs/7129-cargo-audit-advisory-drift.md`

## Error Semantics

Audit failures remain hard failures. Unknown-severity, unwaived, or
above-threshold advisories must continue to stop CI with structured reason
codes. No fallback or silent waiver is permitted.

## Test Plan

### RED

Run `cargo audit --json` and the repository cargo-audit policy checker against
the current lockfile. Confirm `RUSTSEC-2026-0190` and `RUSTSEC-2026-0204`
produce the recorded policy failure.

### GREEN

Run these approved precise updates:

```bash
cargo update -p anyhow --precise 1.0.103
cargo update -p crossbeam-epoch --precise 0.9.20
```

Confirm the lockfile diff contains no unrelated packages, then rerun the audit
policy checker.

### REFACTOR

Review the diff for lockfile-only minimality. Do not introduce manifests,
waivers, helper scripts, or abstractions.

### INTEGRATION

Run `cargo metadata --locked`, the repository audit-policy tests, and a manual
`ci-fast-gate` workflow dispatch on the issue branch. Require the workflow to
pass the audit policy step and execute downstream Rust gates.
