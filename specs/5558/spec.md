# Issue #5558 Spec - PRD Phase-1 kamn-agent-lib Foundation Implementation

- Status: Reviewed
- Issue: #5558
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
`docs/prd/e2e-live-testing-prd.md` defines `kamn-agent-lib` as the critical-path foundation for MCP server, CLI, and E2E harness execution modes, but the crate and all phase-1 modules are absent from the workspace.

## Scope
In scope:
- Add new workspace crate `crates/kamn-agent-lib`.
- Implement phase-1 module files listed in PRD section 13: `identity`, `auth`, `envelope`, `client`, `kolme`, `nonce`, `errors`, and top-level `lib` with `KamnAgentHandle`.
- Add integration tests for auth roundtrip, envelope construction, and proof verification using current KAMN SDK/service primitives.
- Add PRD gap research artifact capturing deterministic phase-1 coverage baseline.

Out of scope:
- CI workflow changes (`.github/workflows/**`).
- New external dependencies beyond crates already in the workspace lock unless separately approved.
- MCP server, CLI, and E2E harness binaries (follow-up issues under #5557).

## Acceptance Criteria
- AC-1: Workspace includes `crates/kamn-agent-lib` with all phase-1 files from PRD section 13 and compiles.
- AC-2: `KamnAgentHandle` exposes ergonomic methods covering identity bootstrap, message send/query, channel/task/escrow operations, health check, and proof verification adapters.
- AC-3: RED->GREEN integration tests exist and pass for auth roundtrip, envelope construction, and proof verification contracts.
- AC-4: Documentation artifacts reflect phase-1 implementation coverage (`docs/research/` and PRD status markers).
- AC-5: Quality gates pass for touched surface (`cargo fmt --check`, `cargo clippy -p kamn-agent-lib -- -D warnings`, targeted tests).

## Conformance Cases
- C-01 (AC-1): crate path and module files exist exactly as required for phase-1.
- C-02 (AC-1): workspace builds `kamn-agent-lib` successfully.
- C-03 (AC-2): handle operations delegate to typed service/auth/proof helpers with deterministic error taxonomy.
- C-04 (AC-3): auth roundtrip test proves signature/header generation contract compatibility.
- C-05 (AC-3): envelope construction test proves stable envelope/signature/nonce behavior.
- C-06 (AC-3): proof verification test proves Kolme verification adapter behavior against current interfaces.
- C-07 (AC-4): phase-1 gap analysis and implementation status markers are present and internally consistent.
- C-08 (AC-5): fmt, clippy, and targeted tests are green.

## Success Metrics / Observable Signals
- `kamn-agent-lib` builds and tests pass as independent workspace crate.
- PRD phase-1 required files are no longer missing.
- Future phase-2/3 crates can depend on a stable shared agent library without duplicating auth/envelope/proof logic.
