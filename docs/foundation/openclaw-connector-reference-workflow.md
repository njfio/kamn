# OpenClaw Flagship Connector and Reference Workflow (Issue #190)

This document captures the first implementation slice for integrating OpenClaw as a flagship agent connector on top of the TypeScript SDK.

## Scope Delivered
- Added `OpenClawConnector` in `packages/kamn-sdk/src/openclaw_connector.ts`.
- Exported connector surface from `packages/kamn-sdk/src/index.ts`.
- Added tests in `packages/kamn-sdk/tests/openclaw_connector.test.ts`.

## Connector Contract
- `registerOpenClawAgent(modelFamily)`:
  - registers a KAMN agent with required OpenClaw capability profile.
  - required capabilities include `openclaw` and `code`.
- `runReferenceWorkflow(request)`:
  - validates prompt and compensation inputs.
  - validates target agent has `openclaw` capability.
  - executes deterministic request-to-settlement flow:
    1. send canonical message
    2. create + accept task
    3. create + release escrow
  - returns workflow IDs (`messageId`, `taskId`, `escrowId`) and `workflowStatus: settled`.

## Validation and Error Handling Rules
- Empty prompt is rejected.
- Compensation must be a positive integer.
- Workflow target must expose `openclaw` capability.
- Unknown agents and invalid SDK paths propagate explicit `SDKError`.

## Fast and Cost-Effective Validation
The connector test lane is dependency-light and runs via Node native TS stripping:

```bash
npm --prefix packages/kamn-sdk test
```

No package install is required at this stage, keeping PR validation fast and low-cost.

## Local Validation
Run from repository root:

```bash
npm --prefix packages/kamn-sdk test
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test openclaw_connector_docs
cargo test -p kamn-core
```
