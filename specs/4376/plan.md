# Plan — #4376

## Approach
- Extend `check_local_kamn_live_runtime_integration_policy.py` runtime command validation with
  in-memory provider marker detection.
- Keep reason naming consistent with existing real-node profile checker taxonomy.
- Update docs where #4371 requires provider rejection reason references.

## Risks
- Risk: inconsistency between general policy checker and real-node checker.
  - Mitigation: use shared marker string and reason-code naming.

## Interfaces
- Marker: `InMemoryKolmeRuntimeCommitClient`
- Reason: `runtime_commit_in_memory_provider_reference_detected`

## ADR
- Not required.
