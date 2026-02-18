# Issue #5018 Spec

- Title: Task: M2 ship DID gateway authn/authz, RLS policy set, and audit log path
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M2 requires a deterministic access gateway contract that authenticates requesters via DID-bound
credentials, authorizes message visibility through ABAC policy rules, emits PostgreSQL RLS policy
templates, and records every gateway decision in an append-only audit trail. The current codebase
has DID parsing and policy engines, but no unified M2 gateway contract surface that combines
authn/authz, scope enforcement, and immutable access logging.

PRD mapping:
- Section 9.1 (Authentication Flow)
- Section 9.2 (Authorization Model / ABAC)
- Section 9.3 (Audit Logging and Access Tracing)
- Section 17/18 scenario 66 (RLS cross-agent denial)

## Acceptance Criteria
- AC-1: DID authentication service validates requester DID shape + credential payload and issues
  bounded session tokens with deterministic token identifiers and expiry metadata.
- AC-2: ABAC authorizer enforces fail-closed message visibility rules for agent participant,
  owner-supervisor, and escrow-auditor access paths; unauthorized access is rejected with stable
  reason codes.
- AC-3: RLS policy templates for `messages` and `access_log` are generated with mandatory
  `kamn.requester_did` session-variable checks and fail-closed predicates when requester context is missing.
- AC-4: Access audit records are append-only, hash-chained, and verifiable for tamper detection.
- AC-5: Shell/workflow/python LOC remains unchanged for this issue (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M2 module in `kamn-core` that provides:
  - DID session token service,
  - ABAC authorization engine for message metadata scope,
  - RLS policy definition generator,
  - append-only access audit ledger contracts.
- Deterministic tests for C-01..C-06 in `kamn-core`.
- Public API exports for follow-on M3+ integration.

Out of scope:
- Live HTTP gateway endpoints and DB wiring.
- SQL migrations and real PostgreSQL `CREATE POLICY` execution.
- Dependency additions or protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid DID auth request with bounded TTL | Session token issued with deterministic token id and expiry |
| C-02 | AC-1 | Unit | Invalid DID / empty credential / invalid TTL | Auth request rejected fail-closed with typed error |
| C-03 | AC-2 | Conformance | ABAC checks for participant, owner, escrow auditor, unrelated requester | Authorized cases pass, unrelated requester denied with stable reason |
| C-04 | AC-3 | Contract | Render M2 RLS policy definitions | Policies include `kamn.requester_did` guard and fail-closed predicates |
| C-05 | AC-4 | Regression | Append audit records then tamper hash chain | Untampered chain verifies; tampered chain fails verification |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/workflow/python/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M2 gateway contracts are exported via `kamn_core` and ready for M3+ integration.
- Shell-to-Rust posture improves/neutral with zero shell LOC increase.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m2_gateway_access` failed before implementation with unresolved `DataLayerM2*` symbols.
- GREEN: `cargo test -p kamn-core --test data_layer_m2_gateway_access` passed (`5 passed, 0 failed`).
- Regression:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
  - `cargo test -p kamn-core` passed.
- Shell-surface marker:
  - `shell_loc_delta_actual: 0`
  - `rust_loc_delta_actual: +768`
  - `shell_to_rust_ratio_delta_actual: improved`
