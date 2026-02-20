# Issue #5315 Plan

## Approach
1. Capture baseline shell/rust ratio metrics before mitigation.
2. Add a failing conformance test that rejects large shell surface in `check_shell_rust_ratio_guardrail.sh`.
3. Move ratio-guardrail logic into `check_shell_rust_ratio_guardrail.py` and keep `.sh` as a thin delegating entrypoint.
4. Re-run ratio-guardrail parity checks and #4000 fixture-matrix checks.
5. Recompute shell/rust ratio metrics and confirm combined #4000 + #5315 deltas are non-positive.

## Affected Modules
- `scripts/ci/check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/check_shell_rust_ratio_guardrail.py`
- `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/test_generate_performance_smoke_report.sh` (regression verification only)
- `specs/5315/{spec,plan,tasks}.md`

## Risks and Mitigations
- Risk: ratio guardrail semantics diverge after language split.
  - Mitigation: run `test_check_shell_rust_ratio_guardrail.sh` parity checks.
- Risk: #4000 fixture behavior regresses indirectly.
  - Mitigation: rerun `test_generate_performance_smoke_report.sh`.
- Risk: mitigation claim lacks measurable proof.
  - Mitigation: capture explicit pre/post ratio metrics and shell LOC file deltas.

## Interfaces and Contracts
- `check_shell_rust_ratio_guardrail.sh` remains the public CLI entrypoint.
- `.sh` entrypoint delegates to `check_shell_rust_ratio_guardrail.py` with identical CLI flags and markers.
- Shell/ratio guardrail contracts remain enforced via existing CI scripts.
