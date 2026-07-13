# Issue 7116: Fix Parallel Preflight Fixture Collision

## Objective

Make agent transaction preflight fixtures unique within a test process so
parallel tests cannot delete or overwrite each other's files.

## Inputs And Outputs

Input: Concurrent construction of preflight fixtures in one test binary.

Output: A distinct temporary root for every fixture construction.

## Boundaries And Non-Goals

- Change test fixture naming only.
- Do not change production preflight validation or error semantics.
- Do not add dependencies or serialize the whole test suite.

## Failure Modes

- Two fixtures receive the same path at coarse filesystem clock resolution.
- The missing-key negative removes a key used by another test.
- The uniqueness fix makes fixture names nondeterministic across processes in a
  way that permits collision.

## Acceptance Criteria

- [x] Parallel fixture construction cannot reuse a root in one process.
- [x] All three preflight contracts pass in parallel.
- [x] Five repeated parallel contract runs pass.
- [x] Formatting and strict targeted clippy pass.

## Files To Touch

- `crates/kamn-e2e-harness/tests/agent_transaction_demo_preflight_contract.rs`
- This spec only.

## Error Semantics

Production public errors remain unchanged. Fixture setup failures continue to
panic with explicit test context.

## Test Plan

RED is the reproduced parallel failure where the pass test reports
`AGENT_TRANSACTION_AGENT_CONFIG_INVALID: required external file is unavailable`.

GREEN:

```bash
for i in 1 2 3 4 5; do
  cargo test -q -p kamn-e2e-harness \
    --test agent_transaction_demo_preflight_contract
done
cargo clippy -p kamn-e2e-harness \
  --test agent_transaction_demo_preflight_contract -- -D warnings
```
