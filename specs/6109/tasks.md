# Tasks: Issue #6109

## Ordered Tasks
- T1 (RED): Add `kamn-core` DID key-binding tests (generate/verify/mismatch/missing binding).
- T2 (GREEN): Implement `AgentDid` key-binding fingerprint generation + verification helpers.
- T3 (RED): Add service-api auth tests for key-binding enforcement in DID-key-map mode.
- T4 (GREEN): Enforce DID/public-key binding in auth and add deterministic reason code taxonomy entry.
- T5 (VERIFY): Run `fmt`, `clippy`, and targeted `kamn-core` + `kamn-node` tests.

## Tier Mapping
- Unit: T1, T2, T3, T4
- Functional: N/A (no route surface change)
- Conformance: T1, T3, T5
- Integration: N/A (module-level auth contract)
- Regression: T5
- Property: N/A (deterministic bounded derivation)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no snapshots)
- Fuzz: N/A (no new parser target in this issue scope)
- Mutation: N/A (workspace mutation gate managed in CI)
- Performance: N/A (no perf contract change)
