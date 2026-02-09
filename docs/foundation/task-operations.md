# Task Operations Command Surface and Snapshot Durability (Issue #128 / #472 / #573 / #617)

This document defines task operation command handling and durable snapshot
persistence/recovery contracts for the task operation registry.

## Core Types
- `TaskOperationEngine`: deterministic in-memory task operations handler.
- `TaskOperationRecord`: task-level operation context:
  - `task_id`
  - `requester`
  - `assignee`
  - `description`
  - `lifecycle` (`TaskLifecycle`)
- `TaskOperationNoticeKind`:
  - `Submitted`
  - `Accepted`
  - `Delegated`
  - `Started`
  - `InputRequired`
  - `Blocked`
  - `Completed`
  - `Failed`
  - `Cancelled`
- `SwarmTaskDraft`: swarm DAG registration payload:
  - `task_id`
  - `requester`
  - `description`
  - `dependencies`
- Snapshot contracts:
  - `TaskOperationRecordSnapshot`
  - `TaskOperationSnapshot`
  - `TaskOperationSnapshotStore`
  - `InMemoryTaskOperationSnapshotStore`
  - `FileTaskOperationSnapshotStore`

## Command Behavior
- `submit(task_id, requester, description)`:
  - creates a new task record in `Submitted`.
  - emits `Submitted` notice.
- `accept(task_id, actor)`:
  - transitions lifecycle using `Accept`.
  - binds `assignee = actor`.
  - emits `Accepted` notice.
- `delegate(task_id, actor, delegatee)`:
  - requires current assignee actor.
  - transitions via `Delegate`.
  - updates assignee to delegatee.
  - emits `Delegated` notice.
- `start_work(task_id, actor)`:
  - assignee-only.
  - transitions via `StartWork`.
  - emits `Started` notice.
- `block(task_id, actor, reason)`:
  - assignee-only with non-empty reason.
  - transitions via `Block`.
  - emits `Blocked` notice.
- `request_input(task_id, actor, reason)`:
  - assignee-only with non-empty reason.
  - transitions via `RequestInput`.
  - emits `InputRequired` notice.
- `complete(task_id, actor)`:
  - assignee-only.
  - transitions via `Complete`.
  - emits `Completed` notice.
- `fail(task_id, actor, reason)`:
  - assignee-only with non-empty reason.
  - transitions via `Fail`.
  - emits `Failed` notice.
- `cancel(task_id, actor)`:
  - requester or current assignee.
  - transitions via `Cancel`.
  - emits `Cancelled` notice.
- `submit_swarm_tasks(drafts)`:
  - registers a bounded DAG-linked task set in a single deterministic pass.
  - rejects duplicate task IDs, duplicate dependency edges, unknown dependency references, and cyclic graphs.
  - initializes dependency metadata used by readiness checks.
- `ready_tasks()`:
  - returns deterministic ready-task IDs (sorted) where lifecycle state is `Accepted` or `Delegated` and all dependencies are `Completed`.
- `export_snapshot()`:
  - returns deterministic snapshot payload with schema version, task records, notices, lifecycle history, and dependency metadata.
- `restore_snapshot(snapshot)`:
  - validates schema version, lifecycle history, dependency references, and cycle safety before mutating engine state.
  - rejects tampered restore payloads where dependency-complete invariants are violated (`Regression: #502`).
- Snapshot store APIs:
  - `TaskOperationSnapshotStore::write(snapshot)`
  - `TaskOperationSnapshotStore::read_latest()`
  - `FileTaskOperationSnapshotStore::recover_latest_and_repair()`

## Validation and Safety Rules
- Task IDs must be unique.
- DIDs must parse as `kamn:did:agent:*`.
- Unauthorized actors are rejected with explicit required-role context.
- Underlying illegal or terminal lifecycle transitions bubble as typed lifecycle errors.
- Swarm dependency rules:
  - dependency IDs must reference registered tasks.
  - cyclic dependency graphs are rejected with `CyclicDependency`.
  - `start_work` is blocked when any dependency is not `Completed` (`DependencyNotSatisfied`).
  - replayed completion attempts remain rejected by terminal-state lifecycle guards (`Regression: #472`).
- Snapshot recovery rules:
  - schema version mismatch is rejected.
  - lifecycle history shape must be replayable from `Submitted`.
  - dependency references must resolve and remain acyclic.
  - tasks restored in execution states (`InProgress`/`InputRequired`/`Blocked`/`Completed`/`Failed`) require dependencies already `Completed` (`Regression: #502`).

## Snapshot Persistence and Restore Contract Rules
- Snapshot schema is versioned by `TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION`.
- File-backed snapshot payload lines are deterministic:
  - `schema|<version>`
  - `task|<task_id>|<requester>|<assignee>|<description>|<history_codes>|<dependency_csv>|<notice_codes>`
- Serialization rejects unsupported delimiters in scalar fields (`|`, newline, carriage return).
- Restore guards are validated via `TaskOperationEngine::restore_snapshot(...)` before file writes are committed.
- Corrupt payload recovery truncates invalid data and returns `latest=None` with `repaired=true`.
- Regression contract:
  - duplicate task IDs on restore are rejected (`Regression: #617`)
  - malformed snapshot payloads are rejected (`Regression: #617`)
  - dependency-completion tampering remains rejected during restore (`Regression: #502`)

## Bounded Graph Benchmark
- A bounded graph benchmark keeps CI cost low while validating DAG guard performance characteristics.
- The benchmark covers a 128-task linear DAG registration path and enforces a generous local CI budget.
- Run bounded graph benchmark lane:
  - `cargo test -p kamn-core --test swarm_task_dag`
- A snapshot roundtrip benchmark validates export+restore overhead for a bounded 128-task DAG without requiring expensive integration infrastructure.
- A bounded snapshot-store roundtrip benchmark validates write/read overhead in PR lanes.

## Federated Delegation Settlement Evidence Contract (Issue #754)
Cross-network task delegation and settlement envelopes must emit deterministic evidence before release approval.

- Stable shell wrappers:
  - `scripts/task/generate_federated_delegation_settlement_evidence_bundle.sh`
  - `scripts/task/check_federated_delegation_settlement_policy.sh`
- Shared Python implementation:
  - `scripts/task/federated_delegation_settlement_contract.py`
- Evidence bundle generator:
  - `bash scripts/task/generate_federated_delegation_settlement_evidence_bundle.sh --output-file /tmp/federated-delegation-settlement.json --delegation-id delegation-go-001 --task-id task-go-001 --delegator-did kamn:did:agent:delegator-go-001 --delegatee-did kamn:did:agent:delegatee-go-001 --source-network kolme-mainnet-a --destination-network kolme-mainnet-b --settlement-reference-id settlement-ref-go-001 --expected-settlement-reference-id settlement-ref-go-001 --settlement-receipt-finality FINAL --nonce-monotonic true --replay-detected false --partition-sequence-monotonic true --required-attestors 2 --received-attestors 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/task/check_federated_delegation_settlement_policy.sh --bundle-file /tmp/federated-delegation-settlement.json`
- PR fast contract lane:
  - `bash scripts/task/run_federated_delegation_settlement_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json federated-delegation-settlement-report.json`
- Partition replay matrix runner:
  - `python3 scripts/task/run_federated_delegation_settlement_matrix.py --fixture fixtures/federated_task_delegation/partition_replay_cases.json --output-json federated-delegation-settlement-report.json`
- Regression policy:
  - settlement reference drift, replay attempts, and tampered final decisions force `NO-GO` (`Regression: #734`).

## Fast and Cost-Effective Validation
Run targeted checks from repository root:

```bash
cargo test -p kamn-core --lib task_operations::tests::
cargo test -p kamn-core --test task_operations
cargo test -p kamn-core --test task_operation_snapshot
cargo test -p kamn-core --test task_operations_docs
bash scripts/task/run_task_operation_snapshot_contract_lane.sh
bash scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh
bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh
bash scripts/task/test_run_federated_delegation_settlement_matrix.sh
bash scripts/task/test_run_federated_delegation_settlement_deep_lane.sh
```

Scheduled deep-lane command:

```bash
cargo test -p kamn-core --lib task_operations::tests::performance_task_operation_snapshot_store_deep_lane_stress -- --ignored
bash scripts/task/run_task_operation_snapshot_deep_lane.sh
bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json federated-delegation-settlement-report.json
```

Then run strict gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```
