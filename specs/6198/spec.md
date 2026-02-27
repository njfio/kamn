# Spec: Issue 6198 - CLI Help/Usage Contract Surface

- Issue: #6198
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

`kamn-cli` rejects `--help` with `unsupported command`, leaving no discoverable command
surface for operators.

## Scope

In scope:
1. Accept `help`, `--help`, and `-h` as a first-class command.
2. Return deterministic usage, command inventory, and flags in both text and JSON output.
3. Preserve existing command parsing behavior for non-help commands.

Out of scope:
1. Full positional-argument parser redesign.
2. Per-command long-form help docs.

## Acceptance Criteria

### AC-1 Help Flags Parse
Given CLI args containing `--help` or `-h`,
When parser runs,
Then parsed command is `Help`.

### AC-2 Help Dispatch Emits Usage Surface
Given parsed help command,
When dispatch executes,
Then output contains usage string, command list, and flags list in both text and JSON.

### AC-3 Existing Commands Unchanged
Given non-help command invocations,
When parser/dispatch runs,
Then pre-existing command behavior remains unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): `lib::tests::regression_issue_6198_cli_parser_accepts_help_flag_as_command`
- C-02 (AC-2, Unit): `lib::tests::regression_issue_6198_cli_dispatch_renders_usage_surface`
- C-03 (AC-3, Integration): `command_activation_contract::spec_c05_cli_core_message_and_task_commands_execute_and_validate_args`
