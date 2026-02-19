# Issue #5133 Plan

- Issue: #5133
- Status: In Progress

## Approach
1. Replace constant-expression assertions with equivalent runtime checks using `std::hint::black_box` in both failing tests.
2. Re-run strict clippy commands used by CI and targeted tests.
3. Run shell-surface guardrails and finalize closure markers.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Budget checks could become opaque if rewritten poorly.
- Mitigations:
  - Keep the same thresholds and error messages.
  - Use minimal local variable indirection only to satisfy lint semantics.

## Interface Contract
- Test-only Rust changes.
- No runtime API/protocol/dependency changes.

## ADR
- Not required.
