# Spec — Issue #3972

- Title: Subtask: implement shell-surface decline trajectory checker with deterministic fail codes
- Parent: #3967
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

The current combined shell-surface trend policy enforces static delta/ratio thresholds, but it does not enforce a time-bounded decline trajectory window. A stale threshold/baseline window can allow prolonged non-improving shell-surface behavior.

## Objective

Extend the combined shell-surface trend policy checker to enforce explicit decline-window and threshold-staleness controls with deterministic reason codes and CI contract coverage.

## Scope

In scope:
- Add decline-window configuration keys to combined shell-surface thresholds.
- Enforce warn/fail when positive shell-surface deltas persist beyond configured windows.
- Enforce fail-closed threshold staleness checks.
- Add deterministic reason codes and test coverage for new paths.
- Update CI strategy docs for new decline-window governance markers.

Out of scope:
- Automatic baseline refresh and auto-remediation.
- Non-shell surface decline policy.

## Acceptance Criteria

- AC-1: Combined trend checker supports explicit decline-window config (`threshold_refreshed_on`, `warn_non_declining_window_days`, `fail_non_declining_window_days`, `threshold_max_age_days`).
- AC-2: Checker emits deterministic reason codes when non-declining trajectory exceeds warn/fail windows.
- AC-3: Checker emits deterministic fail reason when threshold metadata is stale or invalid.
- AC-4: Functional/integration/regression tests validate new decline-window and staleness paths.
- AC-5: `docs/ci/strategy.md` documents decline-window policy markers and commands.

## Conformance Cases

- C-01 (AC-1): Valid threshold file with fresh `threshold_refreshed_on` passes schema/value checks.
- C-02 (AC-2): Positive deltas + exceeded warn window yields `policy_decision=WARN` and `combined_shell_surface_decline_window_warn_exceeded`.
- C-03 (AC-2): Positive deltas + exceeded fail window yields `policy_decision=NO-GO` and `combined_shell_surface_decline_window_fail_exceeded`.
- C-04 (AC-3): Stale threshold metadata yields fail with `combined_shell_surface_threshold_file_stale`.
- C-05 (AC-3): Invalid threshold date/today override yields deterministic invalid-date reason code.
- C-06 (AC-4): CI policy test lane remains green with new reason taxonomy/csv contract.
- C-07 (AC-5): CI strategy docs contain decline-window marker details.

## Success Metrics

- Decline trajectory and staleness guards are enforced in policy checker output.
- New reason taxonomy markers are deterministic and contract-tested.
- Fast CI tools lane remains green.
