# Issue #7134: Migrate Pi Actor Evidence To Service Receipt Authority

## Objective

Make canonical Pi actor evidence derive authority only from validated
`kamn.mcp.authority-receipt.v1` envelopes and final service projections. Keep
Pi process, request, and response hashes as explicitly labeled transport
provenance that cannot satisfy the actor-authority contract.

## Inputs And Outputs

### Inputs

- A persistent authenticated MCP child for exactly one role and DID.
- Versioned registration and mutation authority envelopes returned by
  `kamn-mcp-server`.
- Raw MCP responses and monotonically increasing request IDs used only for
  transport provenance.
- The role's final participant-private or restricted-public service projection.
- The canonical task handoff digest.

### Outputs

- Actor artifacts use `kamn.mvp.pi-transaction-actor.v2`.
- Every artifact binds the role, DID, MCP child process, contiguous request
  range, `transport_response_digests`, task/transaction/escrow IDs, final
  `receipt_chain_commitment`, public commitment, view scope, and handoff digest.
- Registration authority is retained as the service profile commitment bound to
  the role DID.
- Agent A authority contains ordered service receipts for `task:create`,
  `escrow:fund`, and `escrow:release-authorize`.
- Agent B authority contains ordered service receipts for `task:accept` and
  `task:complete`.
- Agent C authority contains no mutation receipts and its restricted-public
  projection contains no participant receipt details.
- Participant receipt IDs, digests, actions, resources, and resulting states
  match the actor-owned entries in the final service projection exactly.

## Boundaries And Non-Goals

- Do not change service receipt generation, MCP envelope generation, projection
  commitment semantics, settlement, the independent Rust verifier, or demo
  closeout behavior.
- Do not add dependencies, OAuth, remote MCP transport, nonce resumption, or a
  replacement MCP child after a child failure.
- Do not treat profile commitments as mutation receipts or transport hashes as
  service authority.
- Compatibility actor-receipt tools may remain registered for older demos, but
  their local artifacts cannot populate or validate v2 service authority.

## Authority Validation

- Registration must be a `service-profile-commitment` envelope with the exact
  schema, source, tool, actor/resource DID, digest-shaped profile commitment,
  and matching nested service result.
- Canonical mutations must be `service-receipt` envelopes with the exact schema,
  source, tool, registered actor DID, resource, resulting state, receipt ID,
  digest, and matching nested service result.
- The MCP session records a service authority entry only after full validation.
  Failed calls record transport provenance only.
- A successful mutation missing or malforming authority fails the entire MCP
  session. A terminal child/session error remains sticky; the workflow cannot
  create a replacement nonce stream for that role.
- V2 normalization rejects unknown authority fields and requires exact role
  receipt order, unique receipt IDs, and unique ID/digest pairs.
- Participant projection receipt entries must exactly equal the role's service
  authority entries. Agent C must have neither mutation authority nor projected
  receipt entries.

## Failure Modes

- Missing, malformed, or mixed-version MCP authority envelope.
- Fabricated, copied, reordered, duplicated, replayed, or cross-role service
  receipt ID/digest.
- Envelope actor, tool, resource, state, or nested result mismatch.
- Missing or mismatched registration profile commitment.
- Missing final receipt-chain commitment or projection receipt set.
- Participant authority that differs from the final participant projection.
- Agent C receipt authority or participant-private projection leakage.
- Non-contiguous transport request IDs or a replaced MCP child.
- V1 or compatibility-only local receipt evidence supplied as canonical v2.

## Error Semantics

- MCP envelope absence uses `MCP_AUTHORITY_RECEIPT_MISSING`.
- Malformed or mismatched MCP authority uses `MCP_AUTHORITY_RECEIPT_INVALID`.
- Actor/projection disagreement uses `PI_SERVICE_AUTHORITY_MISMATCH`.
- Invalid process/request/hash provenance uses
  `PI_TRANSPORT_PROVENANCE_INVALID`.
- Child loss retains the existing fatal session error and never falls back to a
  replacement child or local receipt.
- Errors hard-fail at the Pi boundary; there is no compatibility fallback.

## Acceptance Criteria

- [x] Every canonical registration and mutation validates its versioned MCP
  authority envelope before the workflow consumes the nested service result.
- [x] Pi stores ordered service receipt IDs/digests as authority and stores local
  hashes only as `transport_response_digests`.
- [x] V2 actor evidence binds identity, exact role receipts, transaction facts,
  final chain/public commitments, view scope, and handoff digest.
- [x] Agent A, B, and C receipt sets exactly match their role and final
  projections without Agent C private leakage.
- [x] Missing, fabricated, copied, replayed, reordered, cross-role, malformed,
  v1, compatibility, and projection-mismatched evidence fails closed.
- [x] A failed MCP child/session remains terminal for its actor session.
- [x] Focused Pi tests and the real workflow integration path pass.
- [x] Formatting and issue-scoped repository checks remain clean.

## Files To Touch

- `.pi/extensions/kamn-mvp/mcp-authority.ts`
- `.pi/extensions/kamn-mvp/mcp-session.ts`
- `.pi/extensions/kamn-mvp/mcp-provenance.ts`
- `.pi/extensions/kamn-mvp/live-task-workflow-support.ts`
- `.pi/extensions/kamn-mvp/pi-transaction-evidence.ts`
- Focused `.pi/extensions/kamn-mvp/*.test.ts` files and the fake MCP fixture.
- Existing compatibility receipt modules only when needed to make their
  non-authoritative status explicit.

## Test Plan

### RED

- Require `McpSession` to reject missing, malformed, wrong-actor, wrong-tool,
  copied-resource, and mixed-version authority envelopes.
- Require workflow evidence to expose service authority separately from
  transport provenance and bind the final v2 projection commitment.
- Require actor verification to reject missing, reordered, copied, cross-role,
  replayed, v1, compatibility-only, and projection-mismatched authority.
- Prove a child crash remains terminal and cannot restart the role session.

### GREEN

- Add the minimal typed envelope parser and session authority tracker.
- Emit and validate the minimal v2 actor artifact using service receipts and the
  final service projection.

### REFACTOR

- Keep envelope parsing, transport provenance, authority/projection matching,
  and artifact persistence single-purpose and within repository size limits.
- Remove v1 runtime-receipt authority from the canonical artifact path.

### INTEGRATION

- Exercise the three-role fake MCP workflow with versioned authority envelopes,
  persistent children, role-private projections, and one shared chain/public
  commitment.
- Run focused Pi tests plus formatting and strict repository validation.

## Verification Evidence

- `node --experimental-strip-types --test .pi/extensions/kamn-mvp/*.test.ts`:
  59 passed, 0 failed.
- `node --check` passed for both fake MCP fixture modules.
- `cargo fmt --all --check` passed.
- `git diff --check` passed.
- `make check` and the narrower all-target `kamn-e2e-harness` Clippy command
  both compiled without diagnostics before stalling after crate checks on the
  known macOS local path. The full strict result is delegated to Ubuntu CI.

The independent Rust artifact verifier still consumes v1 artifacts. Its v2
service-authority migration and canonical demo closeout remain #7135 scope, as
listed in this spec's non-goals.
