# Spec: Issue 6208 - Expose SDK Service Timeout Configuration

- Issue: #6208
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: sdk

## Problem Statement

`kamn-sdk` service transport used a fixed `2s` timeout constant with no runtime
configuration path. This prevented environment-specific tuning.

## Scope

In scope:
1. Add configurable SDK service timeout via environment variable.
2. Preserve `2s` as default when not configured.
3. Fail closed for invalid or non-positive configured timeout values.

Out of scope:
1. CLI flag-based timeout wiring.
2. Per-request timeout overrides.

## Acceptance Criteria

### AC-1 Configurable Timeout
Given `KAMN_SDK_SERVICE_TIMEOUT_SECONDS` is set to a positive integer,
When the service transport resolves timeout,
Then it uses the configured value.

### AC-2 Default Behavior Preserved
Given `KAMN_SDK_SERVICE_TIMEOUT_SECONDS` is not set,
When timeout is resolved,
Then timeout defaults to `2` seconds.

### AC-3 Invalid Inputs Fail Closed
Given timeout env value is empty, non-numeric, or zero,
When timeout is resolved,
Then parsing returns deterministic `SdkError::InvalidInput`.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6208_request_timeout_accepts_configured_positive_value`
- C-02 (AC-2, Unit): `tests::regression_issue_6208_request_timeout_defaults_when_env_missing`
- C-03 (AC-3, Unit): `tests::regression_issue_6208_request_timeout_rejects_zero_or_non_numeric_values`

