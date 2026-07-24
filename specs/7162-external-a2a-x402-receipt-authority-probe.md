# 7162 External A2A x402 Receipt-Authority Probe

## Objective

Determine whether the supplied public A2A/x402 contractor exposes
independently verifiable bindings across request, payment challenge, approval,
settlement, and service result without spending funds or using credentials.

## Inputs / Outputs

Inputs:
- `https://a2a.elonsusk.com/.well-known/agent-card.json`
- `https://a2a.elonsusk.com/.well-known/x402.json`
- `https://a2a.elonsusk.com/openapi.json`
- an unpaid request to one advertised x402 endpoint
- an optional documented demo-only approval that cannot move real assets

Outputs:
- a redacted protocol observation fixture
- a deterministic conformance verdict
- an operator-readable validation note
- an opt-in live no-funds probe

## Boundaries / Non-Goals

- Do not pay an invoice or invoke `mark-paid`.
- Do not use a real wallet, private key, API token, or production credential.
- Do not claim external assertions are KAMN service authority.
- Do not claim the external service is secure, production-ready, or compliant
  beyond the exact observed exchange.
- Do not retain arbitrary service output or personal information.
- Do not require live network access for offline regression tests.

## Failure Modes

- Discovery and challenge disagree on network, asset, amount, or payee.
- A challenge lacks a stable request, quote, or payment-requirement identifier.
- Approval is not bound to the challenge and original request.
- Settlement is not bound to approval, challenge, request, and service result.
- A digest is present but cannot be recomputed from defined canonical fields.
- The service returns only ambient actor evidence or opaque assertions.
- The no-funds boundary prevents observing settlement.
- A captured artifact includes secrets or authorization material.

## Verdict Semantics

- `PASS`: every required stage is observed and joined by recomputable
  cryptographic bindings or an equivalent independently verifiable proof.
- `FAIL`: an observed stage omits, contradicts, or breaks a required binding.
- `BLOCKED`: settlement cannot be observed without crossing the no-funds or
  no-credentials boundary. Discovery and challenge findings remain reportable.

## Acceptance Criteria

- [ ] Public discovery and challenge response shapes are captured with UTC
      timestamps and secret-safe redaction.
- [ ] The validator checks request, quote/challenge, approval, settlement,
      service result, payer, payee, asset, network, and amount bindings.
- [ ] Any claimed digest is recomputed from explicitly defined canonical fields.
- [ ] The live probe performs no payment and uses no production credential.
- [ ] Missing evidence yields `FAIL` or `BLOCKED`, never inferred trust.
- [ ] The validation note records observed evidence and exact non-claims.
- [ ] Offline contract tests remain deterministic without network access.

## Files To Touch

- `specs/7162-external-a2a-x402-receipt-authority-probe.md`
- `docs/validation/external-a2a-x402-receipt-authority-probe.md`
- `docs/validation/evidence/7162-external-a2a-x402-observation.json`
- `crates/kamn-e2e-harness/tests/external_a2a_x402_receipt_authority_contract.rs`

## Error Semantics

The offline validator hard-fails malformed fixtures, inconsistent discovery and
challenge fields, invalid digest encodings, or a verdict unsupported by the
observed stages. Network errors in the opt-in live probe produce `BLOCKED` with
an explicit reason; they never fall back to a passing fixture.

## Test Plan

Red:
- Add a focused contract requiring the evidence fixture and validation note.
- Confirm it fails before either artifact exists.

Green:
- Perform the bounded live discovery and unpaid challenge.
- Store only the normalized, redacted observations.
- Add the validation note with the evidence-supported verdict.
- Make the focused offline contract pass.

Refactor:
- Keep canonical field and digest checks centralized in the focused contract.
- Remove redundant raw response fields and verify secret hygiene.

Integration:
- Re-run the no-funds live request and compare invariant protocol fields.
- Run the focused offline contract and the existing authority contracts.
