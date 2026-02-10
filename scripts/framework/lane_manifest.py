#!/usr/bin/env python3
"""Lane manifest parser/validator and phase runner helpers."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
from typing import Any, Mapping
from dataclasses import dataclass

MANIFEST_SCHEMA_VERSION = "kamn.contract-lane.manifest.v1"


@dataclass(frozen=True)
class LaneManifest:
    """Validated lane manifest model."""

    schema_version: str
    lane_id: str
    evidence_key: str
    reason_key: str
    phases: dict[str, tuple[str, ...]]


def _required_string(
    payload: Mapping[str, Any],
    key: str,
    *,
    allow_empty: bool = False,
) -> str:
    raw_value = payload.get(key)
    if not isinstance(raw_value, str):
        raise ValueError(f"{key} must be a string")
    value = raw_value.strip()
    if not allow_empty and value == "":
        raise ValueError(f"{key} must be a non-empty string")
    return value


def _validate_phases(raw_phases: Any) -> dict[str, tuple[str, ...]]:
    if not isinstance(raw_phases, Mapping) or not raw_phases:
        raise ValueError("phases must be a non-empty mapping")

    phases: dict[str, tuple[str, ...]] = {}
    for raw_phase_name, raw_command in raw_phases.items():
        if not isinstance(raw_phase_name, str) or raw_phase_name.strip() == "":
            raise ValueError("phase names must be non-empty strings")
        phase_name = raw_phase_name.strip()

        if not isinstance(raw_command, list) or len(raw_command) == 0:
            raise ValueError(f"phase '{phase_name}' command must be a non-empty list")

        command_parts: list[str] = []
        for part in raw_command:
            if not isinstance(part, str) or part.strip() == "":
                raise ValueError(f"phase '{phase_name}' command parts must be non-empty strings")
            command_parts.append(part)
        phases[phase_name] = tuple(command_parts)

    return phases


def parse_manifest(payload: Mapping[str, Any]) -> LaneManifest:
    """Parse and validate a lane manifest payload."""
    schema_version = _required_string(payload, "schema_version")
    if schema_version != MANIFEST_SCHEMA_VERSION:
        raise ValueError(
            f"schema_version must be '{MANIFEST_SCHEMA_VERSION}'"
        )

    lane_id = _required_string(payload, "lane_id")
    evidence_key = _required_string(payload, "evidence_key")
    reason_key = _required_string(payload, "reason_key")
    phases = _validate_phases(payload.get("phases"))

    return LaneManifest(
        schema_version=schema_version,
        lane_id=lane_id,
        evidence_key=evidence_key,
        reason_key=reason_key,
        phases=phases,
    )


def load_manifest_file(path: Path) -> LaneManifest:
    """Load and validate a lane manifest JSON file."""
    raw_payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw_payload, Mapping):
        raise ValueError("manifest root must be an object")
    return parse_manifest(raw_payload)


def run_lane_phase(
    manifest: LaneManifest,
    phase: str,
    *,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
) -> tuple[int, str]:
    """Execute a validated phase command and return exit code + merged output."""
    if phase not in manifest.phases:
        raise ValueError(f"phase '{phase}' is not defined in manifest '{manifest.lane_id}'")

    phase_command = list(manifest.phases[phase])
    run_env = os.environ.copy()
    if env is not None:
        run_env.update(env)

    result = subprocess.run(
        phase_command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd) if cwd is not None else None,
        env=run_env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")
