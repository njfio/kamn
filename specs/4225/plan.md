# Plan — #4225

Status: Implemented

## Approach
- Locate current async concurrency lane/checker tests.
- Add failing fixture cases for queue-budget and in-flight-budget tamper paths.
- Ensure tests assert explicit deterministic reason markers.

## Risks / Mitigations
- Risk: flakiness from non-deterministic budget timing.
  - Mitigation: use static fixture payloads and deterministic checker inputs.
