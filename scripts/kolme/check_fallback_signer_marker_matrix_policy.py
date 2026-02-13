#!/usr/bin/env python3
"""Validate fallback signer marker retirement matrix policy contracts."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.kolme.fallback-signer-marker-matrix.v1"
ALLOWED_CLASSIFICATIONS = {"keep", "deprecate", "remove-target"}
CLASSIFICATION_RANK = {"keep": 0, "deprecate": 1, "remove-target": 2}
ALLOWED_SURFACES = {
    "runner_guard",
    "policy_reason_code",
    "summary_field",
    "contracts_field",
    "docs_marker",
}
ISSUE_PATTERN = re.compile(r"^#\d+$")
REQUIRED_MARKER_IDS = {
    "runtime_commit_fallback_private_key_command_marker_detected",
    "runtime_signer_fallback_private_key_present_violation",
    "fallback_signer_secret_present_violation",
}
REQUIRED_FIELDS = {
    "marker_id",
    "marker_value",
    "surface",
    "classification",
    "owner",
    "source_entry",
    "parent_issue",
    "target_issue",
}


def require_non_empty_string(payload: dict[str, Any], key: str, index: int) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"marker[{index}] {key} must be a non-empty string")
    return value.strip()


def validate_marker_record(marker: dict[str, Any], index: int) -> str:
    unknown_fields = sorted(set(marker.keys()) - REQUIRED_FIELDS)
    if unknown_fields:
        raise ValueError(
            f"marker[{index}] contains unknown fields: {', '.join(unknown_fields)}"
        )
    missing_fields = sorted(REQUIRED_FIELDS - set(marker.keys()))
    if missing_fields:
        raise ValueError(
            f"marker[{index}] missing required fields: {', '.join(missing_fields)}"
        )

    marker_id = require_non_empty_string(marker, "marker_id", index)
    marker_value = require_non_empty_string(marker, "marker_value", index)
    surface = require_non_empty_string(marker, "surface", index)
    classification = require_non_empty_string(marker, "classification", index)
    owner = require_non_empty_string(marker, "owner", index)
    source_entry = require_non_empty_string(marker, "source_entry", index)
    parent_issue = require_non_empty_string(marker, "parent_issue", index)
    target_issue = require_non_empty_string(marker, "target_issue", index)

    if surface not in ALLOWED_SURFACES:
        raise ValueError(
            f"marker[{index}] surface must be one of {sorted(ALLOWED_SURFACES)}"
        )
    if classification not in ALLOWED_CLASSIFICATIONS:
        raise ValueError(
            f"marker[{index}] classification must be one of {sorted(ALLOWED_CLASSIFICATIONS)}"
        )
    if not owner.islower():
        raise ValueError(f"marker[{index}] owner must use lowercase handle")
    if "/" not in source_entry:
        raise ValueError(
            f"marker[{index}] source_entry must include a script/doc path segment"
        )
    if not ISSUE_PATTERN.match(parent_issue):
        raise ValueError(f"marker[{index}] parent_issue must match #<id>")
    if not ISSUE_PATTERN.match(target_issue):
        raise ValueError(f"marker[{index}] target_issue must match #<id>")
    if " " in marker_id:
        raise ValueError(f"marker[{index}] marker_id must not contain spaces")
    if not marker_value:
        raise ValueError(f"marker[{index}] marker_value must not be empty")

    return marker_id


def validate_matrix(path: Path) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("matrix payload must be a JSON object")

    schema_version = payload.get("schema_version")
    if schema_version != SCHEMA_VERSION:
        raise ValueError(f"schema_version must be {SCHEMA_VERSION}")

    markers = payload.get("markers")
    if not isinstance(markers, list) or not markers:
        raise ValueError("markers must be a non-empty array")

    seen: set[str] = set()
    for index, marker in enumerate(markers):
        if not isinstance(marker, dict):
            raise ValueError(f"marker[{index}] must be an object")
        marker_id = validate_marker_record(marker, index)
        if marker_id in seen:
            raise ValueError(f"marker_id must be unique: {marker_id}")
        seen.add(marker_id)

    missing_required = sorted(REQUIRED_MARKER_IDS - seen)
    if missing_required:
        raise ValueError(
            f"required marker ids missing: {', '.join(missing_required)}"
        )

    classifications = {marker["classification"] for marker in markers}
    if "keep" not in classifications:
        raise ValueError("at least one keep marker classification is required")
    if "remove-target" not in classifications:
        raise ValueError("at least one remove-target marker classification is required")

    expected_order = sorted(
        markers,
        key=lambda marker: (
            CLASSIFICATION_RANK[marker["classification"]],
            marker["marker_id"],
        ),
    )
    if markers != expected_order:
        raise ValueError(
            "markers must be sorted by classification (keep->deprecate->remove-target) and then marker_id"
        )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate fallback signer marker retirement matrix policy."
    )
    parser.add_argument(
        "--matrix-file",
        required=True,
        help="Path to fallback signer marker matrix JSON fixture.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    matrix_path = Path(args.matrix_file)
    if not matrix_path.is_file():
        print(
            f"fallback signer marker matrix policy failed: matrix file not found: {matrix_path}",
            file=sys.stderr,
        )
        return 1
    try:
        validate_matrix(matrix_path)
    except ValueError as error:
        print(f"fallback signer marker matrix policy failed: {error}", file=sys.stderr)
        return 1

    print("fallback signer marker matrix policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
