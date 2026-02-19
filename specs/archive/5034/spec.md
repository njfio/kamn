# Issue #5034 Spec

- Title: Subtask: M5 vector recall, drift, and anomaly-score regression harness
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5021` requires deterministic validation for vector recall, drift,
and anomaly-score behavior. Existing M5 contracts cover semantic query and
anomaly decisions, but no explicit recall-drift evaluation harness exists for
comparing expected baseline top-k embeddings against current query output.

## Acceptance Criteria
- AC-1: M5 exposes deterministic recall-drift evaluation API for owner-scoped
  top-k semantic results against a baseline embedding-id ranking.
- AC-2: Recall-drift evaluation returns deterministic `Stable`/`Degraded`
  decisions with stable reason markers, recall-at-k, rank-shift, and missing-id
  evidence.
- AC-3: Recall-drift evaluation fails closed for invalid thresholds and empty
  baseline contracts.
- AC-4: Anomaly decision reason markers are stabilized via exported constants
  and existing semantic/anomaly behavior remains deterministic.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add recall-drift contract types/API in
  `data_layer_m5_vector_integration`.
- Add conformance tests for stable/degraded drift outcomes and invalid
  threshold/baseline validation.
- Export stable anomaly reason markers and assert them in tests.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI workflow or shell-script changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Baseline top-k exactly matches current semantic query ranking | Drift decision is `Stable` |
| C-02 | AC-2 | Conformance | Baseline top-k contains missing embedding-id in current ranking | Drift decision is `Degraded` with missing-id evidence |
| C-03 | AC-2 | Conformance | Baseline/current overlap with rank shifts | Report includes deterministic `max_observed_rank_shift` evidence |
| C-04 | AC-3 | Regression | Empty baseline or invalid recall threshold | Fail-closed typed errors |
| C-05 | AC-4 | Regression | Anomaly decision path over threshold and within threshold | Reason markers match exported constants |
| C-06 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m5_vector_integration`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5034.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5034.json`

## Success Metrics
- Recall-drift API provides deterministic regression evidence for top-k ranking
  stability/degradation.
- All M5 conformance cases pass in `data_layer_m5_vector_integration` suite.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
