# Plan — Issue #3973

## Approach

1. Add red tests for shell-rust ratio checker behavior and CI-fast wiring expectations.
2. Implement checker + threshold config using git-tracked non-symlink source counting.
3. Wire checker into CI-fast and CI tools contract lanes.
4. Update CI strategy documentation.
5. Run targeted tests and fast CI-tools regression lane.

## Affected Paths

- `.ci/shell-rust-ratio-guardrail.env`
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: mismatch with legacy ratio metrics used by combined trend policy.
  Mitigation: keep this checker explicit and self-contained; do not alter existing trend checker semantics.

- Risk: accidental CI noise from unstable file discovery.
  Mitigation: use `git ls-files` and skip symlinks for deterministic counting.

- Risk: workflow drift vs contracts.
  Mitigation: update and run workflow wiring contract tests with deterministic markers.

## Interfaces / Contracts

- Checker reason taxonomy: `kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1`.
- Checker report schema: `kamn.ci.shell-rust-ratio-guardrail-report.v1`.

## ADR

- Not required (policy/checker integration update only; no protocol or dependency change).
