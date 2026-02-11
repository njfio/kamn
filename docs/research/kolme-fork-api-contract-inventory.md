# KAMN to kolme_fork API Contract Inventory (Issue #1501)

This inventory maps the current `njfio/kolme_fork` API surface to KAMN integration
assumptions so backend integration work can proceed with explicit, fail-closed scope.

## kolm_fork Base API Surface

Source of truth: `packages/kolme/src/api_server.rs` in `njfio/kolme_fork`.

| Endpoint | Method | Observed request contract | Observed response contract |
|---|---|---|---|
| `/healthz` | `GET` | no params | plain text `"Healthy!"` |
| `/fork-info` | `GET` | required query `chain_version=<version>` | JSON `{ "first_block": <int>, "last_block": <int> }` |
| `/get-next-nonce` | `GET` | required query `pubkey=<public-key>` | JSON `{ "next_nonce": <nonce>, "account_id": <nullable account id> }` |
| `/broadcast` | `PUT` | JSON signed transaction payload | JSON `{ "txhash": <hash> }` on success |
| `/block/{height}` | `GET` | path parameter `height` | JSON object with chain/block metadata, tx hash, and logs (not provider/tx_hashes list shape) |
| `/notifications` | `GET` (websocket upgrade) | websocket upgrade handshake | stream of typed notification payloads (`NewBlock`, `FailedTransaction`, `LatestBlock`) |

## Current KAMN Expectations

| KAMN area | Current contract assumption | Alignment status |
|---|---|---|
| Local API probe | `GET /healthz`, `GET /fork-info?chain_version=...` | aligned |
| Native parity harness nonce | `GET /get-next-nonce?pubkey=...` | aligned |
| Runtime commit live provider submit | `POST /broadcast/runtime-commit` with text payload + `X-Idempotency-Key` | mismatch |
| Runtime commit finality checker | `GET /runtime-commit/status?commit_id=...` or `/commit/finality` JSON receipt | mismatch |
| Block fallback reconciler | `/block/{height}` response with `provider`, `block_height`, `tx_hashes` / `failed_tx_hashes` | mismatch |
| Native parity harness broadcast sample | default payload `{"message":"...","signature":"...","recovery_id":1}` | mismatch with signed transaction payload expected by `/broadcast` |

## Gap Report

- Gap: runtime_commit_submit_endpoint_mismatch
  - KAMN assumes `/broadcast/runtime-commit`; kolm_fork exposes `PUT /broadcast`.
- Gap: runtime_commit_payload_shape_mismatch
  - KAMN runtime-commit wire payload is `text/plain` key/value pairs; kolm_fork expects signed transaction JSON.
- Gap: runtime_commit_finality_endpoint_missing
  - KAMN assumes `/runtime-commit/status` or `/commit/finality`; kolm_fork does not expose these endpoints.
- Gap: block_fallback_schema_mismatch
  - KAMN fallback parser expects `provider` + `tx_hashes` schema; kolm_fork `/block/{height}` has a different JSON structure.

## Follow-up Tasks

- #1502 Align runtime commit submit path with kolm_fork `/broadcast` contract.
- #1503 Implement kolm_fork-compatible finality via notifications and block fallback.
- #1504 Build local-only live conformance matrix against kolm_fork endpoints.

## Regression Guard

KAMN-to-kolme_fork endpoint/method/payload contract inventory remains synchronized with code-level integration assumptions (`Regression: #1501`).
