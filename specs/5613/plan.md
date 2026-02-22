# Issue #5613 Plan - External Preflight Rejects Non-File Binary Paths

## Approach
1. Add RED tests for non-file binary path rejection in `sdk-direct` and `mcp-tau` external paths.
2. Implement regular-file checks in preflight with deterministic diagnostics.
3. Add docs artifact + milestone tracking updates.
4. Run required quality gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/r52_preflight_non_file_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-r52-preflight-non-file-diagnostics.md` (new)
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: behavior drift from current deterministic diagnostics.
  - Mitigation: assert explicit error substrings in RED/GREEN tests.
- Risk: regressions in prior harness contracts.
  - Mitigation: full package + cross-crate regression suite.

## Interfaces / Contracts
- Preflight returns deterministic errors:
  - `external execution preflight failed: kolme binary path is not a file: <path>`
  - `external execution preflight failed: agent binary path is not a file: <path>`

## ADR
- Not required (incremental preflight hardening).
