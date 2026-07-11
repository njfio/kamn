# Add Server-Generated Disclosure Projections

## Objective

Generate participant-private and restricted-public transaction views from the
authenticated KAMN service API. Replace harness-authored disclosure claims with
runtime responses derived from the same durable task, escrow, and settlement
records.

## Inputs/Outputs

Signed `GET /v1/tasks/{task_id}/participant-view` requests identify the caller
from the verified sender DID. Only the task creator or assigned provider may
receive a participant response. Signed
`GET /v1/tasks/{task_id}/verifier-view` requests return an allowlisted public
projection to a registered caller with task-read scope.

Both outputs include a schema version, task ID, task state, escrow ID, escrow
state, settlement network, settlement signature when confirmed, commitment,
and one deterministic public commitment over those shared fields. Participant
output additionally includes the caller role and one or more real private
transaction fields, including the task transition receipt digest. Verifier
output never serializes participant receipt digests or private payload fields.

## Boundaries/Non-goals

- Reuse the existing signed service API, durable message store, task ownership,
  escrow agreement, and settlement intent records.
- Server serializers own disclosure. The client and E2E harness must not redact
  a participant response to construct verifier evidence.
- No generalized policy engine, field-encryption scheme, anonymous access,
  web UI, or new dependency.
- No Pi orchestration changes in this issue; independent actors consume these
  routes in the dependent issue.
- No mainnet, custody, dispute, refund, or multi-chain claim.

## Failure Modes

- Missing task: existing route-not-found response.
- Unregistered or invalidly authenticated caller: existing authentication
  response.
- Participant caller is neither creator nor assigned provider:
  `TASK_PARTICIPANT_VIEW_FORBIDDEN` (403).
- Task has no bound escrow: `TASK_ESCROW_BINDING_MISSING` (409).
- Durable task, escrow, or settlement records disagree:
  `TRANSACTION_PROJECTION_INCONSISTENT` (500).
- Persistence failure: existing observable state-persistence error (500).

All failures are hard failures. A verifier view cannot fall back to a generated
placeholder, client-provided commitment, or participant response.

## Acceptance Criteria

- [ ] Participant and verifier response types and serializers are separate.
- [ ] Participant membership is derived from authenticated server context.
- [ ] Unrelated and unregistered agents cannot retrieve participant output.
- [ ] Verifier output is built from an explicit allowlist of durable fields.
- [ ] Participant output contains at least one real field absent from verifier
      output.
- [ ] Verifier output contains no participant receipt or private payload digest.
- [ ] Shared fields and deterministic public commitment match across views.
- [ ] Confirmed devnet settlement fields come from persisted settlement evidence.
- [ ] Signed HTTP integration tests exercise both routes against durable state.
- [ ] The E2E harness can consume runtime responses without synthesizing private
      or public digest labels.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/models.rs` or a focused projection
  model module
- `crates/kamn-node/src/service_api_endpoint/message_store/` projection reads
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/queries.rs`
  and focused route helpers
- service API task/escrow authorization integration contracts
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_views.rs` only for runtime
  response consumption and artifact persistence

## Error Semantics

Interior projection code returns structured reason codes with task ID and
requester DID context. It does not log. The HTTP route catches once, logs via
the existing request boundary, and returns the stable status/code pair. Missing
or inconsistent durable state never produces a partial projection.

## Test Plan

RED:

- An unrelated registered Agent C can retrieve participant-private output.
- Participant and verifier routes return the same response shape.
- Verifier output exposes a transition receipt or participant-private digest.
- Shared public commitment differs between participant and verifier output.
- Harness-generated view evidence passes without a runtime projection response.

GREEN:

- Add focused projection models and one deterministic shared-field commitment.
- Add store reads that join task, bound escrow, and settlement evidence.
- Add signed participant and verifier query routes with explicit authorization.
- Feed runtime responses into the existing three-agent proof artifact boundary.

REFACTOR:

- Keep files below 200 lines and functions below 25 lines.
- Centralize shared-field projection and commitment generation without merging
  participant and verifier serializers.
- Remove synthetic digest construction made obsolete by runtime responses.

INTEGRATION:

- Run signed HTTP participant/verifier contracts, full `kamn-node`, full
  `kamn-e2e-harness`, formatting, strict workspace clippy, and `make check`.
- Run `make demo-mvp` and canonical verification. This issue proves server
  disclosure behavior; it does not claim independent Pi execution yet.
