# Escrow Settlement Slice

This runbook documents one current KAMN service-API slice on `main` that proves service-api escrow lifecycle persistence through fund, release, and restart-visible released state. It is deliberately narrower than external chain settlement or bridge finality claims.

## Scope
- task create and accept on the current service-api path
- escrow fund and release on the current service-api path
- released escrow state remaining queryable after restart

## Proof Anchors
This slice is grounded in current-main tests:
- `integration_service_api_endpoint_persists_task_and_escrow_state_across_routes`
- `integration_service_api_endpoint_persists_task_and_escrow_state_across_restart`

## What This Proves
- the service API can create and accept a task before funding escrow on the same persisted state path
- the service API can fund escrow and return `state=funded`
- the service API can release escrow and return `state=released`
- released escrow state remains persisted across restart on the current service-api storage path
- the proof is anchored to executable route and restart coverage, not API shape alone

## What This Does Not Prove
- not bridge finality
- not external chain settlement
- not Byzantine-safe or adversarial settlement
- not cross-node escrow consensus
- not production deployment readiness

## Operator Commands
Run the exact escrow proof targets from a clean checkout:

```bash
cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_routes -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_restart -- --exact --nocapture
```

## Expected Evidence
- route-level evidence shows one task entering `accepted`, one escrow entering `funded`, and the same escrow reaching `released`
- restart-level evidence shows persisted task state remains `accepted` and persisted escrow state remains `released`

## Notes
- This slice proves service-api escrow lifecycle persistence.
- It does not prove external chain receipt finality or cross-chain settlement semantics.
