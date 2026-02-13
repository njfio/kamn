#!/usr/bin/env python3
"""Validate Kolme lane migration matrix schema and deterministic policy rules."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.kolme.lane-migration-matrix.v1"
ALLOWED_PRIORITIES = {"P0", "P1", "P2"}
PRIORITY_RANK = {"P0": 0, "P1": 1, "P2": 2}
ALLOWED_CURRENT_RUNNERS = {
    "legacy_bash_lane",
    "hybrid_shell_python",
    "framework_manifest_lane",
}
REQUIRED_TARGET_RUNNER = "framework_manifest_lane"
ALLOWED_STATUSES = {"planned", "in_progress", "migrated"}
ISSUE_PATTERN = re.compile(r"^#\d+$")
REQUIRED_LANE_IDS = {
    "kolme.version.compatibility",
    "kolme.runtime.commit.adapter",
    "kolme.runtime.commit.replay",
    "kolme.notifications.consumer",
    "kolme.block.fallback.reconciliation",
    "kolme.managed_signer_backend_slo.policy",
    "kolme.managed_signer_backend_slo.telemetry",
    "kolme.nonce.broadcast.parity",
    "kolme.local.fork.rust_matrix",
    "kolme.local.kamn.live_runtime_integration",
    "kolme.local.heavy.validation_matrix",
}
REQUIRED_FIELDS = {
    "lane_id",
    "owner",
    "priority",
    "current_runner",
    "target_runner",
    "status",
    "source_entry",
    "parent_issue",
    "target_issue",
}


def require_non_empty_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{key} must be a non-empty string")
    return value.strip()


def validate_lane_record(lane: dict[str, Any], index: int) -> str:
    unknown_fields = sorted(set(lane.keys()) - REQUIRED_FIELDS)
    if unknown_fields:
        raise ValueError(
            f"lane[{index}] contains unknown fields: {', '.join(unknown_fields)}"
        )
    missing_fields = sorted(REQUIRED_FIELDS - set(lane.keys()))
    if missing_fields:
        raise ValueError(
            f"lane[{index}] missing required fields: {', '.join(missing_fields)}"
        )

    lane_id = require_non_empty_string(lane, "lane_id")
    owner = require_non_empty_string(lane, "owner")
    priority = require_non_empty_string(lane, "priority")
    current_runner = require_non_empty_string(lane, "current_runner")
    target_runner = require_non_empty_string(lane, "target_runner")
    status = require_non_empty_string(lane, "status")
    source_entry = require_non_empty_string(lane, "source_entry")
    parent_issue = require_non_empty_string(lane, "parent_issue")
    target_issue = require_non_empty_string(lane, "target_issue")

    if priority not in ALLOWED_PRIORITIES:
        raise ValueError(
            f"lane[{index}] priority must be one of {sorted(ALLOWED_PRIORITIES)}"
        )
    if current_runner not in ALLOWED_CURRENT_RUNNERS:
        raise ValueError(
            f"lane[{index}] current_runner must be one of {sorted(ALLOWED_CURRENT_RUNNERS)}"
        )
    if target_runner != REQUIRED_TARGET_RUNNER:
        raise ValueError(
            f"lane[{index}] target_runner must be {REQUIRED_TARGET_RUNNER}"
        )
    if status not in ALLOWED_STATUSES:
        raise ValueError(
            f"lane[{index}] status must be one of {sorted(ALLOWED_STATUSES)}"
        )
    if not ISSUE_PATTERN.match(parent_issue):
        raise ValueError(f"lane[{index}] parent_issue must match #<id>")
    if not ISSUE_PATTERN.match(target_issue):
        raise ValueError(f"lane[{index}] target_issue must match #<id>")
    if "/" not in source_entry:
        raise ValueError(
            f"lane[{index}] source_entry must include a script path segment"
        )
    if not owner.islower():
        raise ValueError(f"lane[{index}] owner must use lowercase handle")

    return lane_id


def validate_matrix(path: Path) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("matrix payload must be a JSON object")

    schema_version = payload.get("schema_version")
    if schema_version != SCHEMA_VERSION:
        raise ValueError(f"schema_version must be {SCHEMA_VERSION}")

    lanes = payload.get("lanes")
    if not isinstance(lanes, list) or not lanes:
        raise ValueError("lanes must be a non-empty array")

    lane_ids: list[str] = []
    seen: set[str] = set()
    for index, lane in enumerate(lanes):
        if not isinstance(lane, dict):
            raise ValueError(f"lane[{index}] must be an object")
        lane_id = validate_lane_record(lane, index)
        if lane_id in seen:
            raise ValueError(f"lane_id must be unique: {lane_id}")
        seen.add(lane_id)
        lane_ids.append(lane_id)

    missing_required = sorted(REQUIRED_LANE_IDS - seen)
    if missing_required:
        raise ValueError(
            f"required lane ids missing: {', '.join(missing_required)}"
        )

    expected_order = sorted(
        lanes,
        key=lambda lane: (
            PRIORITY_RANK[lane["priority"]],
            lane["lane_id"],
        ),
    )
    if lanes != expected_order:
        raise ValueError(
            "lanes must be sorted by priority (P0->P2) and then lane_id"
        )

    if all(lane["status"] == "migrated" for lane in lanes):
        raise ValueError(
            "at least one lane must remain planned or in_progress while migration is active"
        )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate Kolme lane migration matrix policy."
    )
    parser.add_argument(
        "--matrix-file",
        required=True,
        help="Path to lane migration matrix JSON fixture.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    matrix_path = Path(args.matrix_file)
    if not matrix_path.is_file():
        print(
            f"kolme lane migration matrix policy failed: matrix file not found: {matrix_path}",
            file=sys.stderr,
        )
        return 1
    try:
        validate_matrix(matrix_path)
    except ValueError as error:
        print(f"kolme lane migration matrix policy failed: {error}", file=sys.stderr)
        return 1

    print("kolme lane migration matrix policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
