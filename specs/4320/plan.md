# Issue #4320 Plan

- Issue: `#4320`
- Status: `Completed`

## Approach
- Introduce peer adapter reason projection types/functions in `p2p_transport` for deterministic reason-code normalization.
- Introduce deterministic multi-process validation hook contracts for process-isolated peer lanes.
- Add a dedicated test suite covering required categories (unit/functional/integration/regression/performance).
- Update release go/no-go checklist and docs contract tests with peer reason taxonomy references.

## Affected Modules
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/tests/p2p_peer_adapter_reason_projection.rs` (new)
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: projection taxonomy drift with existing reason-code strings.
- Mitigation: map directly to existing deterministic reason literals and assert exact values in tests.
- Risk: hook list drift causes lane instability.
- Mitigation: fixed hook ordering plus deterministic marker assertions in integration/regression tests.

## Interface Contract
- Additive API only; no wire/protocol change.
- Existing `P2pTransportError` reason-code behavior remains unchanged.

## ADR
- Not required (no dependency/architecture/protocol change).
