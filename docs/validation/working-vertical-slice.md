# Working Vertical Slice

This runbook documents one current KAMN slice on `main` that proves a coherent service-API flow with two identities, encrypted delivery evidence, a task lifecycle transition, and audit export output.

## Scope
- Runtime surface: `kamn-node` service API in processor/api mode.
- Identities: one sender DID and one recipient/worker DID.
- Delivery path: real service-api message send, relay projection, mailbox query, and message query.
- Task path: real task creation followed by a real task lifecycle transition to `completed` through dispatch on task query.
- Evidence path: sender state snapshot, audit export bundle, and `data_layer_runtime_evidence` persisted on the message record.

## Preconditions
- Clean checkout on current `main`.
- Rust toolchain able to build `kamn-node`.
- No external services are required for this slice.

## Run
```bash
cargo test -p kamn-node integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence -- --nocapture
```

## Expected Evidence
- two identities are present in the flow: sender DID and recipient/worker DID
- encrypted delivery is evidenced by `data_layer_runtime_evidence` and the canonical encryption algorithm marker `X25519-XChaCha20-Poly1305`
- recipient mailbox query returns the relayed message and the message query reports `delivered`
- task lifecycle transition advances to `completed`
- audit export contains a `service_api_task_created` record for the created task

## What This Proves
- One current KAMN service-api path can carry a message between two identities.
- The delivery path persists `data_layer_runtime_evidence` alongside the message record.
- The same runtime surface can register a worker, create a task, and complete one real task lifecycle transition.
- The runtime emits operator-verifiable audit export output for the task portion of the flow.

## What This Does Not Prove
- It does not prove wire-level ciphertext interoperability beyond the persisted encryption contract/evidence path.
- It does not prove bridge finality, Byzantine settlement, or multi-node consensus.
- It does not prove production deployment readiness or fault tolerance.
