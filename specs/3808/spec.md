# Issue #3808 Spec

- Title: `Subtask: add signer extraction threshold and ownership budget guards`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
After signer decomposition, `signer.rs` can regress into monolithic growth without deterministic budget and ownership guardrails.

## Scope
In:
- Add signer extraction budget contract test for `signer.rs` line-count threshold.
- Add ownership marker checks for signer submodule routing.
- Add CI strategy documentation for running guard command.

Out:
- Non-signer budget policy.
- Runtime behavior changes.

## Acceptance Criteria
- AC-1: Given signer extraction budget policy, when contract tests run, then monolith regrowth past threshold fails closed.
- AC-2: Given signer ownership routing, when contract tests run, then required module declarations/re-exports are present.
- AC-3: Given CI strategy docs, when docs checks run, then signer extraction guard command/policy markers remain documented.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `cargo test -p kamn-node --test signer_extraction_budget_contract signer_module_budget_stays_within_threshold -- --exact --nocapture` | `signer.rs` LOC stays within configured threshold |
| C-02 | AC-2 | Regression/Conformance | `cargo test -p kamn-node --test signer_extraction_budget_contract signer_module_declares_required_extraction_ownership_markers -- --exact --nocapture` | signer module routing markers remain intact |
| C-03 | AC-3 | Docs/Conformance | `cargo test -p kamn-node --test signer_extraction_budget_contract docs_ci_strategy_declares_signer_extraction_budget_guard_policy -- --exact --nocapture` | CI strategy docs include signer extraction budget guard markers/command |

## Test Mapping
- C-01/C-02/C-03: `crates/kamn-node/tests/signer_extraction_budget_contract.rs`

## Success Metrics
- signer extraction budget check is deterministic and CI-friendly.
- ownership marker drift fails closed.
- CI strategy explicitly documents guard policy and command.
