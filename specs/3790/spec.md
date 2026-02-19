# Issue #3790 Spec

- Title: Subtask: implement transient classifier and bounded retry schedule for Kolme HTTP transport
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Shared Kolme HTTP transport retry behavior needs deterministic transient classification and bounded retry scheduling contracts that are explicitly documented and fail closed on drift.

## Acceptance Criteria
- AC-1: Transient classifier marks timeout/unavailable provider errors as retryable with deterministic reason codes.
- AC-2: Bounded deterministic retry schedule remains capped and deterministic across attempts.
- AC-3: Non-transient provider errors are fail-closed/no-retry with deterministic terminal decisions.
- AC-4: Runtime architecture docs include a transient classifier + bounded retry schedule contract table, pinned by docs-contract tests.
- AC-5: Unit, Functional, Integration, and Regression evidence is present and passing (Integration may be N/A with written justification for helper-only scope).

## Scope
In scope:
- `crates/kamn-node/src/runtime_kolme_live.rs` helper behavior and helper tests
- `docs/architecture/kolme-runtime-commit.md` transient classifier table contract
- `crates/kamn-node/tests/kolme_runtime_commit_docs.rs` docs-contract drift assertions
- `specs/3790/{spec.md,plan.md,tasks.md}`

Out of scope:
- Runtime policy redesign beyond helper contract boundaries
- New shell governance tooling or CI workflow modifications
- Protocol/wire-format changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `classify_retry_category` with `Timeout` and `Unavailable` provider errors | Returns deterministic retry categories `timeout` and `unavailable` |
| C-02 | AC-2 | Unit | `deterministic_retry_backoff_millis` and jittered variant across attempt range | Backoff sequence is deterministic and capped at max |
| C-03 | AC-3 | Functional | `retry_decision_for_attempt` matrix for transient/non-transient examples | Transient retries until ceiling; malformed response remains fail-fast |
| C-04 | AC-3 | Regression | malformed-response fail-fast regression helper test | Malformed response remains non-retry and explicit fail-closed |
| C-05 | AC-4 | Integration/Regression | docs-contract tests over `docs/architecture/kolme-runtime-commit.md` | Required transient classifier/schedule markers exist; missing markers fail test |
| C-06 | AC-5 | Integration | helper-only slice | N/A with explicit justification in tasks/PR matrix |

## Test Mapping
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_classifier_marks_transient_provider_errors -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_backoff_policy_is_deterministic_and_bounded -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_decision_matrix_respects_attempt_ceiling_contract -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::unit_retry_backoff_with_jitter_stays_bounded_and_deterministic -- --exact`
- `cargo test -p kamn-node runtime_kolme_live::tests::regression_retry_classifier_keeps_malformed_fail_fast -- --exact`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3790.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3790.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3790.json`

## Success Metrics
- Runtime helper and docs contract stay deterministic and fail closed for retry taxonomy drift.
- Non-transient malformed provider responses remain no-retry fail-fast.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
