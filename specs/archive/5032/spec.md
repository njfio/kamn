# Issue #5032 Spec

- Title: Subtask: M3 blind-index correctness and search determinism regression corpus
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5019` delivered baseline M3 blind-index and metadata search
contracts. The remaining high-risk gap is an explicit deterministic regression
corpus contract that compares expected baseline message ordering to current
owner-scoped blind-index query output.

## Acceptance Criteria
- AC-1: M3 exposes deterministic blind-index determinism-evaluation API for
  owner-scoped exact-match queries against baseline ordered message IDs.
- AC-2: Determinism evaluation returns deterministic `Stable`/`Drifted`
  decisions with stable reason markers and mismatch evidence (missing,
  unexpected, out-of-order IDs).
- AC-3: Determinism evaluation fails closed for invalid baseline contracts
  (empty baseline IDs or invalid query limit).
- AC-4: Existing blind-index/metadata search behavior remains deterministic and
  passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add determinism evaluation contract types/API in
  `data_layer_m3_blind_index_search`.
- Add conformance tests for stable/drifted and fail-closed baseline validation.
- Export stable determinism reason-marker constants and assert them in tests.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI/workflow/shell-surface modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Baseline order exactly matches current query output | Determinism decision is `Stable` |
| C-02 | AC-2 | Conformance | Baseline has missing/unexpected/out-of-order IDs vs current query | Determinism decision is `Drifted` with evidence fields populated |
| C-03 | AC-3 | Regression | Empty baseline IDs or zero limit | Fail-closed typed errors |
| C-04 | AC-4 | Regression | Existing M3 blind-index/metadata conformance cases | Existing deterministic behavior remains green |
| C-05 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m3_blind_index_search`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5032.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5032.json`

## Success Metrics
- Determinism evaluation report provides deterministic regression evidence for
  blind-index query output drift.
- All M3 conformance cases pass in `data_layer_m3_blind_index_search` suite.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
