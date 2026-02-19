# Issue #3941 Plan

- Issue: #3941
- Status: In Progress
- Spec: `specs/3941/spec.md`

## Implementation Approach
1. Add a signer-source regression test that detects `unreachable!()` via a constructed marker (to avoid self-matching literals).
2. Run the new regression test first to capture RED.
3. Replace the remaining `unreachable!()` branch in signer decode-failure assertions with explicit typed-error assertions.
4. Update runtime watchdog attestation docs with panic-path retirement mapping for this subtask.
5. Re-run the mapped scoped tests for GREEN + regression verification.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `docs/foundation/runtime-watchdog-attestation.md`

## Risks and Mitigations
- Risk: source-string regression test becomes brittle.
  - Mitigation: scope check to signer module and only enforce panic primitive marker.
- Risk: decode-failure assertion weakens type guarantees.
  - Mitigation: assert typed variant first, then inspect message from typed variant.

## Contracts and Interfaces
- Decode failure contract remains `ConfigError::RuntimeKolmeLive` with deterministic message content.
- Signer-source regression contract enforces no `unreachable!()` marker in module source.

## Verification Strategy
- RED: run new signer-source regression before removing the macro.
- GREEN: remove `unreachable!()` branch and keep typed assertions.
- REGRESSION: run signer decode-failure, startup panic-control-flow, and runtime watchdog docs contract tests.
