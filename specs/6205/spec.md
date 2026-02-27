# Spec: Issue #6205 - Replace Pipe-Delimited Snapshot Journal Records with Serde JSON

- Status: Implemented
- Priority: P2
- Parent: #6183
- Milestone: R59 Swarm Gap Closure

## Problem Statement

Snapshot journal records currently rely on custom pipe-delimited parsing. This format is brittle
and duplicates manual parsing logic. The journal format should be normalized to serde-backed JSON
records with explicit schema markers.

## Scope

In scope:
- Replace pipe-delimited snapshot journal line format with serde JSON lines.
- Preserve fail-closed parsing behavior for corrupt tail entries.
- Keep snapshot replay behavior intact for message/channel/task stores.

Out of scope:
- Historical backfill migration tooling for old journal files.
- Broader snapshot schema redesign beyond line format replacement.

## Acceptance Criteria

### AC-1 Serde JSON Journal Records
Given snapshot journal append operations,
When new entries are written,
Then each line is a serde-serialized JSON record with schema marker.

### AC-2 Fail-Closed Corruption Handling
Given malformed journal entries,
When replay is attempted,
Then parser returns `None`/error and existing callers emit corrupt-tail reason codes.

### AC-3 Replay Compatibility for Current Stores
Given valid snapshots in message/channel/task stores,
When replay executes,
Then snapshots restore successfully.

## Conformance Cases

- C-01 (AC-1, Unit): snapshot journal helper tests assert JSON-line write/parse markers.
- C-02 (AC-2, Regression): malformed line tests fail closed in helper and callers.
- C-03 (AC-3, Functional): targeted lifecycle snapshot tests pass for message/channel/task flows.
