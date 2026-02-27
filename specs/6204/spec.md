# Spec: Issue #6204 - Begin kamn-core Split with Snapshot Journal Extraction

- Status: Implemented
- Priority: P2
- Parent: #6183
- Milestone: R59 Swarm Gap Closure

## Problem Statement

`kamn-core` remains oversized and cross-domain. A practical first split step is to extract shared
snapshot-journal persistence helpers into a focused crate consumed by `kamn-core` domain modules.

## Scope

In scope:
- Create a new focused crate for snapshot-journal helpers.
- Move shared snapshot-journal logic out of `kamn-core`.
- Rewire `message_lifecycle`, `channel_models`, and `task_operations` to use the extracted crate.

Out of scope:
- Full multi-crate decomposition of all `kamn-core` domains.
- Public API redesign outside snapshot-journal consumers.

## Acceptance Criteria

### AC-1 Focused Extraction
Given workspace crates,
When members are enumerated,
Then a dedicated snapshot-journal crate exists.

### AC-2 Core Dependency Wiring
Given `kamn-core` builds,
When snapshot stores compile,
Then they import journal helpers from the extracted crate (not local duplicate module).

### AC-3 Regression Safety
Given lifecycle snapshot persistence tests,
When targeted suites execute,
Then behavior remains green.

## Conformance Cases

- C-01 (AC-1, Functional): new crate exists in workspace and exports snapshot journal helpers.
- C-02 (AC-2, Unit): `kamn-core` snapshot modules compile against extracted crate imports.
- C-03 (AC-3, Regression): targeted `kamn-core` snapshot-related tests stay green.
