# Issue #5236 Spec

- Title: Subtask: Restore block-pipeline typed-DID fixture compatibility after wave C
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
Wave-C typed DID enforcement tightened listener identity parsing across runtime quorum paths. Block-pipeline integration tests still use legacy fixture identifiers (`kamn:did:listener:alpha`), causing deterministic CI failures in `block_pipeline` and `block_pipeline_transport_fed` despite no production behavior regression.

## Scope
In:
- `crates/kamn-core/tests/block_pipeline.rs`
- `crates/kamn-core/tests/block_pipeline_transport_fed.rs`
- `specs/5236/{spec.md,plan.md,tasks.md}`

Out:
- Relaxing `AgentDid` parser contracts.
- Runtime/prod logic changes outside test fixtures.
- Shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 20
- shell_to_rust_ratio_delta_estimate: -0.0001
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: `block_pipeline` quorum commit-path test uses valid typed listener DID fixtures and passes deterministically.
- AC-2: `block_pipeline_transport_fed` commit-path/fork-choice/stale-candidate/performance tests use valid typed listener DID fixtures and preserve existing expected outcomes.
- AC-3: Regression verification includes failing-target reproduction and green rerun for both failing test targets.
- AC-4: Shell-surface delta remains zero.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `block_pipeline` functional commit test with typed DID listener fixture | Commit succeeds without listener DID parse error |
| C-02 | AC-2 | Integration | `block_pipeline_transport_fed` fork-choice and stale-candidate scenarios with typed DID fixtures | Existing fork-choice/stale outcomes remain unchanged |
| C-03 | AC-3 | Regression | `cargo test -p kamn-core --test block_pipeline --test block_pipeline_transport_fed` | Both targets pass in same run |
| C-04 | AC-4 | Conformance | Shell/rust ratio guard check | No shell LOC increase |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test block_pipeline --test block_pipeline_transport_fed`
- C-04:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`

## Success Metrics
- Mainline CI `Fast Gate (PR)` no longer fails on block-pipeline typed-DID fixture mismatch.
- Fixture semantics remain explicit and aligned with typed DID contracts.
- Shell-to-rust ratio is neutral or improved.
