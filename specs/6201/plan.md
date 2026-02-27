# Plan: Issue 6201 - Reduce E2E Driver Duplication Surface

- Issue: #6201
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Add a new shared driver helper module under `drivers/` with pure functions.
2. Start with high-confidence duplicated helpers used by all three drivers:
   - env default lookup
   - boolean flag parse
   - live scenario gating
   - replay marker validation
   - percentile index and latency budget checks
3. Replace local helper copies in each driver with shared calls.
4. Add unit tests in the shared helper module.
5. Run scoped formatting/lint/tests for `kamn-e2e-harness`.

## Affected Modules

- `crates/kamn-e2e-harness/src/drivers/mod.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/src/drivers/shared.rs` (new)

## Risks and Mitigations

1. Risk: helper extraction can subtly change probe failure messages.
   - Mitigation: preserve existing message text and run scoped tests.
2. Risk: scenario routing drift when replacing `is_live_bound_scenario_id`.
   - Mitigation: explicit unit test for full `S-01..S-15` live-bound set.

