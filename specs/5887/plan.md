# Plan: Issue #5887

## Approach
1. Add local env-resolution helpers per driver using explicit `match` on `env::var`.
2. Replace direct `unwrap_or`/`unwrap_or_else` env fallback call sites with helpers.
3. Expand panic-path checker `DEFAULT_RUNTIME_ROOTS` to include e2e harness src.
4. Validate with default checker, all-roots checker, and e2e-harness tests.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `scripts/ci/check_no_production_expect.py`

## Risks / Mitigations
- Risk: broad replacements in long driver files may alter scenario defaults.
  - Mitigation: use small helper abstraction preserving exact default values and rerun harness tests.
- Risk: checker expansion surfaces latent violations.
  - Mitigation: run all-roots audit before and after edits and fail closed.

## Interfaces / Contracts
- No API or wire contract changes.
- CI contract change: panic-path checker default coverage expands to include e2e harness source.
