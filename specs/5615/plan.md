# Issue #5615 Plan - External Preflight Requires Absolute Binary Paths

## Approach
1. Add RED tests for relative-path rejection and absolute-path pass behavior.
2. Implement absolute-path checks in preflight with deterministic diagnostics.
3. Add docs artifact + milestone tracking updates.
4. Run required quality gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/r52_preflight_absolute_path_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-r52-preflight-absolute-path-diagnostics.md` (new)
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: order of checks changes existing deterministic error ordering.
  - Mitigation: explicit RED/GREEN assertions for expected error content.
- Risk: regression in prior preflight contracts.
  - Mitigation: full harness and cross-crate regression suite.

## Interfaces / Contracts
- Preflight deterministic errors:
  - `external execution preflight failed: kolme binary path must be absolute: <path>`
  - `external execution preflight failed: agent binary path must be absolute: <path>`

## ADR
- Not required (incremental preflight hardening).
