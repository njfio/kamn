# Issue #5005 Spec

- Title: Story: M2 DID access gateway with ABAC, RLS, and audit trails
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M2 access-gateway contracts for DID authentication, fail-closed ABAC
message visibility authorization, RLS policy templates, and append-only
tamper-evident access auditing. Story delivery is completed through child task
`#5018`.

## Acceptance Criteria
- AC-1: M2 DID authentication and bounded session issuance are implemented with
  deterministic success and fail-closed invalid-input behavior.
- AC-2: M2 ABAC authorization enforces participant/owner/auditor allow paths and
  unrelated-requester deny paths with stable reason markers, including negative
  authorization matrix drift detection.
- AC-3: M2 RLS template contracts and append-only hash-chained access audit
  ledger behavior are implemented and validated by deterministic tests.
- AC-4: Story maps to PRD M2 requirements with reproducible test evidence, and
  shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5018`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD M2 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M2 expansion beyond the accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m2_gateway_access` auth/session tests | Valid DID requests issue bounded deterministic sessions; invalid identity/credential inputs fail closed |
| C-02 | AC-2 | Conformance | Run ABAC allow/deny and negative matrix cases | Participant/owner/auditor allows pass, unrelated access denies with stable reason codes, drift cases are detected |
| C-03 | AC-3 | Regression | Run RLS template + audit hash-chain tests | Policy contracts include requester guard predicates and audit-chain tamper checks pass/fail deterministically |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because
  shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5005` closes with child task `#5018` merged and ACs mapped to passing
  deterministic tests.
- M2 gateway contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
