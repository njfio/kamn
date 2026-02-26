# Plan: Issue #6088

## Approach
1. Create #6088 lifecycle artifacts and lock extraction boundary to `live_probe_matrix`.
2. Execute RED step by asserting extracted-crate test lane fails before crate exists.
3. Create new crate `kamn-live-probe-matrix` and move current `live_probe_matrix` implementation + tests.
4. Replace `kamn-core/src/live_probe_matrix.rs` with compatibility re-export.
5. Wire workspace membership and `kamn-core` dependency.
6. Run targeted regression lanes (`kamn-core` contract test + extracted crate tests), then broader affected crate tests.

## Affected Modules
- `Cargo.toml`
- `crates/kamn-live-probe-matrix/Cargo.toml`
- `crates/kamn-live-probe-matrix/src/lib.rs`
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/live_probe_matrix.rs`
- `crates/kamn-core/tests/live_probe_matrix_contract.rs` (expected unchanged for parity)
- `specs/6088/spec.md`
- `specs/6088/plan.md`
- `specs/6088/tasks.md`
- `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`

## Risks / Mitigations
- Risk: API drift during move breaks downstream imports.
  Mitigation: keep `kamn-core` compatibility re-export and run unchanged contract test lane.
- Risk: missing trait derives/visibility differences after extraction.
  Mitigation: copy implementation verbatim first; refactor only after green parity tests.
- Risk: lane-order contract with #6087.
  Mitigation: keep #6088 scoped and dependency-linked; merge only after #6087 lane confirms landed.

## Interfaces / Contracts
- Public API contract preserved at `kamn_core::LiveProbeMatrix*` symbols.
- New crate interface: `kamn_live_probe_matrix::LiveProbeMatrix*`.
- No wire/protocol/schema change.
