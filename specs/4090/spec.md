# Issue #4090 Spec

- Title: Subtask: define quota policy fixture matrix and parser helper contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Deterministic quota-enforcement behavior depends on stable fixture coverage and parser-helper contracts; without explicit fixture matrix contracts, checker behavior can drift silently.

## Acceptance Criteria
- AC-1: Fixture matrix covers valid and invalid quota windows across scope classes.
- AC-2: Parser helper contracts are deterministic and test-backed.
- AC-3: Unit/Functional/Integration/Regression tests pass for fixture/parser scope.
- AC-4: Ops configuration docs include quota fixture markers and taxonomy references.

## Scope
In scope:
- Add quota fixture matrix file under `fixtures/runtime/`.
- Add Rust contract tests for fixture parser/helper behavior.
- Update `docs/ops/configuration.md` with fixture/taxonomy markers.
- Update docs-contract test assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- `specs/4090/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Dynamic policy distribution.
- Runtime scheduler behavior changes.
- Shell/workflow additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Fixture matrix file with valid/invalid cases | Matrix includes deterministic pass/fail quota-window coverage |
| C-02 | AC-2 | Unit | Parser helper contract functions | Parser deterministically parses metadata/cases and rejects malformed lines |
| C-03 | AC-3 | Integration | Fixture parser contract test binary | All fixture cases evaluate to expected status/reason markers |
| C-04 | AC-4 | Regression | Ops configuration docs-contract assertions | Required quota fixture/taxonomy markers exist and remain stable |

## Test Mapping
- `cargo test -p kamn-core --test quota_policy_fixture_parser_contract`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs -- service_api_ops_configuration_contains_quota_policy_fixture_matrix_controls --exact`

## Success Metrics
- Deterministic quota fixture matrix and parser helper contracts are established.
- No shell LOC growth in this subtask.
