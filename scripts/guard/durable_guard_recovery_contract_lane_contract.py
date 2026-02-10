#!/usr/bin/env python3
"""Durable guard recovery contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/guard/run_durable_guard_recovery_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_command(root_dir: Path, command: list[str]) -> int:
    """Run a command in repository root and return its exit code."""
    result = subprocess.run(
        command,
        cwd=root_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode


def main(argv: list[str]) -> int:
    """Execute durable guard recovery contract-lane tests."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    commands = [
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "unit_delivery_guard_snapshot_rejects_schema_mismatch",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "unit_channel_policy_snapshot_rejects_schema_mismatch",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "functional_delivery_guard_recovery_restores_nonce_and_replay_state",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "functional_channel_policy_recovery_restores_retention_candidates",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "integration_durable_guard_recovery_matrix_restores_delivery_and_retention_invariants",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "regression_corrupted_delivery_snapshot_rejected_with_explicit_error",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "regression_corrupted_channel_snapshot_rejected_with_explicit_error",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_recovery_matrix",
            "performance_durable_guard_recovery_contract_lane_budget",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_snapshot_store",
            "unit_bundle_schema_mismatch_is_rejected",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_snapshot_store",
            "functional_in_memory_bundle_store_roundtrip",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_snapshot_store",
            "integration_file_bundle_restore_preserves_invariants",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_snapshot_store",
            "regression_truncated_bundle_payload_rejected",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "durable_guard_snapshot_store",
            "performance_bundle_contract_lane_budget",
        ],
        ["cargo", "test", "-p", "kamn-core", "--test", "message_delivery_guards_docs"],
        ["cargo", "test", "-p", "kamn-core", "--test", "channel_permissions_retention_docs"],
        ["cargo", "test", "-p", "kamn-core", "--test", "release_gonogo_checklist_docs"],
    ]

    for command in commands:
        exit_code = run_command(root_dir, command)
        if exit_code != 0:
            return fail(f"contract lane command failed: {' '.join(command)}")

    print("durable guard recovery contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
