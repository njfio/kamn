# Plan: Issue #5854

## Approach
1. Preserve runtime behavior while extracting run-path endpoint runtime-mode branching into explicit helpers.
2. Add unit/regression tests over those helpers to deterministically kill operator mutants around `full`/`api` guard decisions.
3. Add one binary integration test that executes the real entrypoint with invalid runtime mode, providing direct fail-closed coverage for `run() -> Ok(())` mutation.
4. Run targeted tests, then run scoped in-diff mutation lane for `kamn-node` and resolve any residual misses.

## Affected Modules
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/main_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*` (new run-path mutation guard test file)
- `crates/kamn-node/tests/runtime_entrypoint_mutation_contract.rs` (new integration test)
- `specs/5854/spec.md`
- `specs/5854/plan.md`
- `specs/5854/tasks.md`

## Risks & Mitigations
- Risk: runtime guard refactor could alter existing endpoint behavior.
  - Mitigation: keep helper outputs equivalent to existing branches and validate with targeted tests.
- Risk: integration entrypoint test can become flaky if it depends on external services.
  - Mitigation: use invalid runtime-mode parse failure path only; no network/service dependency.
- Risk: mutation run introduces additional survivors in unrelated lines.
  - Mitigation: limit diff surface and add precise tests for each changed decision branch.

## Interfaces / Contracts
- Service API runtime-mode contract remains:
  - `full` => skip in-process endpoint (supervised in orchestration),
  - `api` => serve in-process endpoint,
  - otherwise fail closed with deterministic error text.
- Observability runtime-mode contract remains:
  - `full` => skip in-process endpoint (supervised in orchestration),
  - otherwise evaluate snapshot + serve path.

## ADR
- Not required (no dependency/protocol/architecture decision; local testability hardening only).
