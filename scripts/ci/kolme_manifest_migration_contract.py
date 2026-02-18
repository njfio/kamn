#!/usr/bin/env python3
"""Validate migrated Kolme manifest-backed contract lane wrappers by group."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

CONFIG_SCHEMA = "kamn.kolme-manifest-migration-contract-groups.v1"
MANIFEST_SCHEMA = "kamn.contract-lane.manifest.v1"
DELETION_MANIFEST_SCHEMA = "kamn.ci.superseded-script-deletion-manifest.v1"
DELETION_MANIFEST_FILE = "fixtures/ci/superseded_script_deletion_manifest.json"
WRAPPER_MARKER = "scripts/framework/run_manifest_lane.sh"


def _read_json(path: Path, *, label: str) -> Any:
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError as exc:
        raise SystemExit(f"expected {label} to exist: {path}") from exc
    except OSError as exc:
        raise SystemExit(f"failed to read {label}: {path}: {exc}") from exc
    try:
        return json.loads(text)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"failed to parse {label} JSON at {path}: {exc}") from exc


def _lane_wrapper_shell_loc(path: Path) -> int:
    if path.is_symlink():
        return 1
    try:
        with path.open("r", encoding="utf-8") as handle:
            return sum(1 for _ in handle)
    except OSError as exc:
        raise SystemExit(f"failed reading lane wrapper for LOC accounting: {path}: {exc}") from exc


def _expect_string(value: Any, *, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise SystemExit(f"expected non-empty string for {label}")
    return value


def _load_deleted_scripts(root_dir: Path) -> set[str]:
    payload = _read_json(root_dir / DELETION_MANIFEST_FILE, label="superseded script deletion manifest")
    if payload.get("schema_version") != DELETION_MANIFEST_SCHEMA:
        raise SystemExit("unexpected superseded script deletion manifest schema version")
    deletions = payload.get("deletions")
    if not isinstance(deletions, list):
        raise SystemExit("expected superseded script deletion manifest deletions array")

    deleted: set[str] = set()
    for index, entry in enumerate(deletions):
        if not isinstance(entry, dict):
            raise SystemExit(f"expected deletion manifest entry object at index {index}")
        script_path = entry.get("script_path")
        if not isinstance(script_path, str) or not script_path:
            raise SystemExit(f"expected deletion manifest script_path string at index {index}")
        # Transitional waves keep some deletion-manifest entries scheduled but not yet removed.
        if not (root_dir / script_path).exists():
            deleted.add(script_path)
    return deleted


def _validate_manifest_contract(
    manifest_path: Path,
    *,
    expected_lane_id: str,
    expected_contract_script: str,
) -> None:
    payload = _read_json(manifest_path, label="manifest file")
    if payload.get("schema_version") != MANIFEST_SCHEMA:
        raise SystemExit("unexpected manifest schema version")
    if payload.get("lane_id") != expected_lane_id:
        raise SystemExit("unexpected lane_id for migrated lane manifest")
    phases = payload.get("phases")
    if not isinstance(phases, dict) or "contract" not in phases:
        raise SystemExit("manifest missing contract phase")
    command = phases["contract"]
    if not isinstance(command, list) or len(command) < 2:
        raise SystemExit("manifest contract phase command must be a non-empty list")
    if command[0] != "python3" or command[1] != expected_contract_script:
        raise SystemExit("manifest contract phase must invoke expected python contract lane script")


def _validate_group(root_dir: Path, config_path: Path, group_key: str, deleted_scripts: set[str]) -> str:
    config = _read_json(config_path, label="Kolme migration config")
    if config.get("schema_version") != CONFIG_SCHEMA:
        raise SystemExit("unexpected Kolme migration config schema version")

    groups = config.get("groups")
    if not isinstance(groups, dict):
        raise SystemExit("expected migration config to contain a groups object")

    group = groups.get(group_key)
    if not isinstance(group, dict):
        raise SystemExit(f"unknown migration group key: {group_key}")

    max_shell_loc = group.get("max_shell_loc")
    if not isinstance(max_shell_loc, int) or max_shell_loc < 1:
        raise SystemExit(f"expected positive integer max_shell_loc for group: {group_key}")

    success_message = _expect_string(group.get("success_message"), label=f"group {group_key} success_message")

    lanes = group.get("lanes")
    if not isinstance(lanes, list) or not lanes:
        raise SystemExit(f"expected non-empty lanes array for group: {group_key}")

    total_shell_loc = 0
    for lane in lanes:
        if not isinstance(lane, dict):
            raise SystemExit(f"expected lane entry object in group: {group_key}")

        lane_script = _expect_string(lane.get("lane_script"), label=f"group {group_key} lane_script")
        manifest_file = _expect_string(lane.get("manifest_file"), label=f"group {group_key} manifest_file")
        expected_lane_id = _expect_string(lane.get("lane_id"), label=f"group {group_key} lane_id")
        expected_contract_script = _expect_string(
            lane.get("contract_script"),
            label=f"group {group_key} contract_script",
        )

        lane_script_path = root_dir / lane_script
        manifest_path = root_dir / manifest_file

        lane_marked_deleted = lane_script in deleted_scripts
        if lane_marked_deleted:
            if lane_script_path.exists():
                raise SystemExit(f"expected superseded lane script to be deleted: {lane_script}")
        else:
            if not lane_script_path.exists():
                raise SystemExit(f"expected migrated lane script to exist: {lane_script}")
            if not lane_script_path.is_file():
                raise SystemExit(f"expected migrated lane script to be a file: {lane_script}")
            if not lane_script_path.stat().st_mode & 0o111:
                raise SystemExit(f"expected migrated lane script to be executable: {lane_script}")

            try:
                lane_script_text = lane_script_path.read_text(encoding="utf-8")
            except OSError as exc:
                raise SystemExit(
                    f"failed to read lane script for routing assertion: {lane_script_path}: {exc}"
                ) from exc

            if WRAPPER_MARKER not in lane_script_text:
                raise SystemExit(
                    f"expected migrated lane script to dispatch through manifest wrapper: {lane_script}"
                )

        if not manifest_path.is_file():
            raise SystemExit(f"expected manifest file for migrated lane: {manifest_file}")

        _validate_manifest_contract(
            manifest_path,
            expected_lane_id=expected_lane_id,
            expected_contract_script=expected_contract_script,
        )

        if not lane_marked_deleted:
            total_shell_loc += _lane_wrapper_shell_loc(lane_script_path)

    if total_shell_loc > max_shell_loc:
        raise SystemExit(
            "expected "
            f"{group_key} migrated shell LOC to stay at or below {max_shell_loc}, got {total_shell_loc}"
        )

    return success_message


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate grouped Kolme manifest-migration CI contracts.",
    )
    parser.add_argument("--root-dir", required=True, help="Repository root directory path.")
    parser.add_argument("--config-file", required=True, help="Path to migration group config JSON.")
    parser.add_argument("--group", required=True, help="Migration group key to validate.")
    return parser


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()

    root_dir = Path(args.root_dir).resolve()
    config_path = Path(args.config_file).resolve()
    group_key = args.group

    deleted_scripts = _load_deleted_scripts(root_dir)
    success_message = _validate_group(root_dir, config_path, group_key, deleted_scripts)
    print(success_message)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
