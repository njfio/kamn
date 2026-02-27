# Plan: Issue #6122

## Approach
1. Add `drivers/shared_helpers.rs` with shared helper functions currently duplicated across all three E2E drivers.
2. Wire `drivers/mod.rs` to expose the shared module at crate scope.
3. Replace duplicated helper implementations in:
   - `drivers/sdk_direct.rs`
   - `drivers/cli_scripted.rs`
   - `drivers/mcp_agent.rs`
4. Keep existing call sites and error wording stable by using thin wrappers only when needed.
5. Run crate-level verification (`fmt`, `clippy`, `test`) and preserve regression test coverage.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/mod.rs`
- `crates/kamn-e2e-harness/src/drivers/shared_helpers.rs` (new)
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`

## Risks
- Risk: subtle message/format drift in helper error paths can break existing assertions.
  - Mitigation: keep helper semantics and message templates unchanged; run full crate tests.
- Risk: module visibility/import changes can break test modules.
  - Mitigation: keep helper names stable in driver module scope when tests rely on `super::...`.

## Interfaces/Contracts
- No public API changes.
- Internal helper ownership moves from per-driver local functions to shared module functions.
