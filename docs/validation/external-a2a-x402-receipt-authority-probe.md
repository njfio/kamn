# External A2A x402 Receipt-Authority Probe

## Verdict: FAIL

The bounded probe observed a valid unpaid x402 challenge, but it did not observe
a request body digest, nonce, challenge ID, or expiry timestamp in that
challenge. The advertised signing hint names a nonce without supplying one.
The challenge therefore does not expose enough information to bind an approval
or later settlement to this exact request.

Settlement visibility is separately `BLOCKED`: no signed retry, payment, real
wallet, credential, or invoice action was permitted by this probe.

## Scope

- Public peer: `https://a2a.elonsusk.com`
- Endpoint: `POST /x402/file.deliver`
- Observation time: `2026-07-24T02:52:35.735Z`
- One unsigned request with no payment or authorization material
- Normalized evidence:
  `docs/validation/evidence/7162-external-a2a-x402-observation.json`

## Observed Exchange

1. Public agent-card, x402 manifest, and OpenAPI bytes returned HTTP `200`.
2. The request used a fixed canonical JSON body and a locally recomputed
   request body digest.
3. The service returned HTTP `402` and payment terms for `solana:demo`.
4. Network, asset, payee, and USD amount matched public discovery.
5. The challenge exposed no nonce or challenge ID, no request body digest, and
   no absolute expiry timestamp.
6. No approval response was observed.
7. No settlement response was observed.
8. No service result was observed.

In exact terms, no approval response was observed; no settlement response was observed.

The hashes in the evidence fixture are observer-computed integrity markers.
They are not issuer receipts and are not KAMN service authority.

## Authority Criteria

A passing peer-authority exchange would need to make these bindings
independently verifiable:

- request payload to quote or challenge
- challenge terms to a stable nonce, identifier, and expiry
- approval to request, challenge, payer, payee, asset, network, and amount
- settlement to the approved challenge and an authoritative chain transaction
- service result to the same settlement receipt

The observed challenge satisfies payment-term consistency but not the first two
bindings. Later stages cannot repair an approval that was not tied to a unique
request and challenge.

## Operator Command

The live request is intentionally a single unsigned POST:

```text
POST https://a2a.elonsusk.com/x402/file.deliver
content-type: application/json

{"prompt":"KAMN no-funds receipt-authority conformance probe"}
```

Expected safe response:

- HTTP `402`
- x402 version `2`
- demo-network payment terms
- no signed retry and no side-effecting service result

## What This Does Not Prove

- not KAMN service authority
- not successful approval or settlement
- not payment correctness or one-transfer idempotence
- not production readiness
- not mainnet safety, custody, or economic finality
- not security of the external service

The result applies only to the exact public exchange and response shape observed
at the timestamp above.
