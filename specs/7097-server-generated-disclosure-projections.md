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

- [x] Participant and verifier response types and serializers are separate.
- [x] Participant membership is derived from authenticated server context.
- [x] Unrelated and unregistered agents cannot retrieve participant output.
- [x] Verifier output is built from an explicit allowlist of durable fields.
- [x] Participant output contains at least one real field absent from verifier
      output.
- [x] Verifier output contains no participant receipt or private payload digest.
- [x] Shared fields and deterministic public commitment match across views.
- [x] Confirmed devnet settlement fields come from persisted settlement evidence.
- [x] Signed HTTP integration tests exercise both routes against durable state.
- [x] The E2E harness can consume runtime responses without synthesizing private
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

## Completion Evidence

- `make check` passed formatting and strict workspace clippy with all targets
  and features.
- `cargo test -p kamn-node` passed the full package suite: 677 unit tests and
  all standalone contract binaries; one explicitly live-only devnet test was
  ignored by its declared environment gate.
- `cargo test -p kamn-e2e-harness` passed the full package suite, including 255
  library tests and the MVP disclosure, receipt, transcript, and verifier
  contract binaries. Explicit local-runtime live probes remained ignored.
- `make demo-mvp` returned `GO` in optional mode, with local-only claims labeled
  separately and production readiness left `NOT_CLAIMED`.
- The funded required-mode run returned `GO`, transferred 1,000,000 lamports on
  Solana devnet, observed finalized balance movement, and persisted the same
  settlement signature reported by the live service API.
- The required-mode run wrote separate runtime-owned
  `runtime-agent-a-participant-view.json`,
  `runtime-agent-b-participant-view.json`, and
  `runtime-agent-c-verifier-view.json` artifacts.
- Canonical verification passed against `.kamn/demo/latest/proof/report.json`
  after both optional and required-mode runs.

## Deferred Boundary

The runtime-owned projection artifacts are additive in this issue. The legacy
canonical three-agent narrative and its generated view artifacts remain in the
proof bundle until the plan's dependent transcript-replacement issue. They do
not replace or manufacture the runtime projection responses, and they must not
be cited as proof that independent Pi actors executed the transaction.
