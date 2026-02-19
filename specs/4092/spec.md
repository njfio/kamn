# Issue #4092 Spec

- Title: Subtask: implement starvation fixture set and fairness checker behavior contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Fairness governance needs deterministic starvation fixtures and checker behavior contracts so starvation classes are detected consistently and cannot drift silently.

## Acceptance Criteria
- AC-1: A deterministic starvation fixture set exists and covers representative starvation classes.
- AC-2: A fairness checker evaluates fixture inputs with fail-closed deterministic reason codes.
- AC-3: Unit, Functional, Integration, and Regression tests verify checker behavior and fixture drift protection.
- AC-4: Ops configuration documentation includes fairness fixture markers and validation command references.

## Scope
In scope:
- Add starvation fixture matrix under `fixtures/runtime/`.
- Add fairness checker Rust API with deterministic reason taxonomy markers.
- Add Rust contract tests that parse fixtures and validate checker behavior.
- Update ops documentation and docs-contract assertions.
- Create `specs/4092/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Scheduler algorithm redesign.
- Shell/workflow script changes.
- CI lane additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Starvation fixture matrix | Fixture includes deterministic starvation/pass classes and metadata markers |
| C-02 | AC-2 | Unit | Fairness checker invalid inputs | Checker rejects unknown scope and invalid fairness bounds with deterministic reason codes |
| C-03 | AC-2 | Integration | Fairness checker + fixture cases | Checker decisions match expected status/reason for all fixture cases |
| C-04 | AC-3 | Regression | Fixture reason taxonomy drift check | Checker reason-code list remains superset of fixture reason-code declarations |
| C-05 | AC-4 | Regression | Ops docs contract assertions | Required fairness fixture markers and command references are present |

## Test Mapping
- `cargo test -p kamn-core --test fairness_policy_checker_contract`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs -- service_api_ops_configuration_contains_fairness_starvation_fixture_controls --exact`

## Success Metrics
- Deterministic starvation fixture coverage is explicit and parser-checker validated.
- No shell LOC growth for this subtask (`shell_loc_delta_actual=0` target).
