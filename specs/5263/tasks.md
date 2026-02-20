# Issue #5263 Tasks

- Issue: #5263
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing tests for Phase-2 pipeline contracts and adapter blind-index persistence insert path.
- [x] T2 (Implementation/GREEN): add Phase-2 pipeline module with deterministic encryption + blind-index derivation output.
- [x] T3 (Implementation/GREEN): extend adapter insert execution to persist provided blind-index map JSON.
- [x] T4 (Regression): validate fail-closed errors for malformed key refs/invalid blind-index inputs/missing recipient mappings.
- [x] T5 (Verification): run fmt, strict clippy, and targeted adapter/bridge/pipeline/public-api suites.
- [x] T6 (Process): update issue/docs/spec status with closure markers and shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | pipeline deterministic output + validation failures |
| Functional | envelope -> ciphertext -> M0 + blind-index artifact generation |
| Integration | adapter insert with derived blind indexes + search retrieval |
| Regression | malformed key/material failure matrix |
| Performance | N/A (Phase-2 operational slice) |
