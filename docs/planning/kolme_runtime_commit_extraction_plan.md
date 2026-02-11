# Kolme Runtime Commit Extraction Plan

## Problem Statement
`crates/kamn-core/src/kolme_runtime_commit.rs` still concentrates transport wiring, finality polling, block-fallback reconciliation, lifecycle orchestration, and adapter glue in one module. This plan defines a staged extraction path into `kamn-kolme` while preserving behavior and keeping validation fast and cost-aware.

## Scope Boundary
In scope:
- move transport-facing parsing/validation helpers from `kamn-core` runtime-commit path into `kamn-kolme`,
- move finality and block-fallback integration helpers into `kamn-kolme` with stable contracts,
- keep `kamn-core` focused on runtime orchestration and product-level error typing.

Out of scope:
- changing public runtime-commit API semantics,
- introducing new network protocols,
- expanding CI runtime with expensive live-network jobs.

## Target Module Boundaries
- `kamn-kolme::transport`: endpoint parsing, request/response framing, websocket handshake/frame parsing.
- `kamn-kolme::finality`: provider-response field extraction and finality alias normalization.
- `kamn-kolme::block_fallback`: block-path rendering, fallback payload parsing, lookup-window and block-identity validation.
- `kamn-core::kolme_runtime_commit`: orchestration, lifecycle transitions, adapter policy, and fail-closed domain errors.

## Phase 1 - Transport and endpoint parsing extraction
- expose transport parsing contracts from `kamn-kolme` and replace remaining core-local transport wrappers,
- keep call-site behavior unchanged with extraction-boundary tests,
- gate completion on strict `clippy` and runtime-commit transport regression tests.

## Phase 2 - Finality and block-fallback extraction
- move finality parsing and block-fallback conversion contracts behind `kamn-kolme` interfaces,
- keep provider mismatch and commit-id mismatch fail-closed behavior in `kamn-core`,
- gate completion on finality and block-fallback regression suites.

## Phase 3 - Adapter and lifecycle orchestration extraction
- split remaining adapter/transport bridge glue into dedicated modules (or subcrate) with explicit ownership,
- preserve lifecycle-state and idempotency semantics in `kamn-core`,
- gate completion on end-to-end runtime pipeline tests and extraction-boundary guard tests.

## Risk Register
- coupling drift between `kamn-core` error variants and extracted `kamn-kolme` contracts,
- silent behavior drift during wrapper removal or inline mapping updates,
- extraction churn increasing review burden unless changes remain small and issue-scoped.

## Validation Matrix
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo test -p kamn-core --test kolme_runtime_commit_extraction_boundary`
- `cargo test -p kamn-core --test kolme_runtime_commit_finality`
- `cargo test -p kamn-core --test kolme_runtime_commit_client`
- `cargo test -p kamn-core --test kolme_runtime_commit_client_docs`

Execution policy:
- keep local runtime-commit test runs as the default validation loop,
- keep CI focused on fast, selector-based checks until extraction stabilizes,
- reserve expensive live-network or forked-backend exercises for opt-in local validation lanes.

## Exit Criteria
- extraction-boundary test no longer detects legacy wrappers for extracted responsibilities,
- runtime-commit behavior remains fail-closed across provider mismatch, malformed response, and timeout paths,
- module ownership is explicit enough to split `kolme_runtime_commit` into maintainable submodules without behavioral regressions.

Regression: #1814
