# Spec — #4392 Subtask: RED Tests for WebSocket Protocol Drift and Invalid Session-Frame Acceptance

Status: Implemented
Priority: P1
Parent: #4387
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Current websocket policy tests do not fully lock deterministic behavior for required-field drift and
normalized failure reason output across protocol/session tamper cases.

## Scope

In scope:
- RED tests for websocket protocol/taxonomy drift.
- RED tests for required-field drift handling.
- RED assertions for normalized `reason_codes_value` output in success/failure paths.

Out of scope:
- Policy checker implementation logic.

## Acceptance Criteria

AC-1: Tests fail when websocket protocol/session drift is undetected.

AC-2: Tests fail when missing required fields are not mapped to deterministic reasons.

AC-3: Tests fail when normalized `reason_codes_value` markers are absent or unstable.

## Conformance Cases

- C-01 (AC-1, Functional): taxonomy version tamper fails with deterministic mismatch reason.
- C-02 (AC-1, Regression): taxonomy CSV tamper fails with deterministic mismatch reason.
- C-03 (AC-2, Regression): missing taxonomy CSV field fails with deterministic required-field reason.
- C-04 (AC-3, Functional): success output includes `reason_codes_value=none`.
- C-05 (AC-3, Regression): failure output includes deterministic `reason_codes_value` reason marker.
