# Plan: Issue #6125

## Approach
1. Identify a coherent low-coupling module slice in `kamn-core` for phase-1 extraction.
2. Add RED boundary contract coverage proving extraction expectations.
3. Create new crate, move slice implementation, and rewire `kamn-core` via dependency + re-export.
4. Run GREEN regression and module verification suites.
5. Update lifecycle artifacts and open PR with AC mapping and TDD evidence.

## Affected Modules
- `crates/kamn-snapshot-journal/src/lib.rs`
- `crates/kamn-snapshot-journal/Cargo.toml`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/tests/core_split_phase1_contract.rs`
- workspace `Cargo.toml` and crate `Cargo.toml` wiring
- `specs/6125/spec.md`
- `specs/6125/plan.md`
- `specs/6125/tasks.md`

## Risks / Mitigations
- Risk: broad dependency fan-out makes extraction non-minimal.
  Mitigation: pick a narrow boundary and preserve API through re-export.
- Risk: hidden runtime coupling causes behavioral drift.
  Mitigation: retain existing tests and add explicit boundary contracts.
- Risk: scope balloons toward full decomposition.
  Mitigation: ship phase-1 only with clear follow-up boundaries.

## Interfaces / Contracts
- Preserve existing snapshot-store behavior and journal line semantics (`entry|1|<payload-hex>`).
- Move shared journal hex/path/record parsing helpers into extracted crate while keeping `kamn-core` error mapping local.
- No wire/protocol changes as part of extraction.
