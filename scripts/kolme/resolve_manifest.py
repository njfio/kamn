#!/usr/bin/env python3
"""Resolve Kolme wrapper metadata from manifest files."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def emit_failure(error_code: str, error_detail: str) -> int:
    print("status=fail")
    print(f"error_code={error_code}")
    print(f"error_detail={error_detail}")
    return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Resolve Kolme wrapper manifest path and phase."
    )
    parser.add_argument(
        "--manifests-dir",
        required=True,
        help="Directory containing lane manifest JSON files.",
    )
    parser.add_argument(
        "--wrapper-name",
        required=True,
        help="Wrapper file name, e.g. run_local_foo_lane.sh.",
    )
    parser.add_argument(
        "--required-phase",
        default="",
        help="Optional phase that the resolved manifest must project.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifests_dir = Path(args.manifests_dir)
    wrapper_name = args.wrapper_name
    required_phase = args.required_phase.strip()

    if not manifests_dir.is_dir():
        return emit_failure(
            "invalid_manifest",
            f"manifest directory does not exist: {manifests_dir}",
        )

    matches: list[tuple[Path, dict[str, Any]]] = []
    for manifest_path in sorted(manifests_dir.glob("kolme_*.json")):
        try:
            payload = json.loads(manifest_path.read_text(encoding="utf-8"))
        except Exception as exc:  # noqa: BLE001
            return emit_failure(
                "invalid_manifest",
                f"unable to parse manifest {manifest_path.name}: {exc}",
            )
        if not isinstance(payload, dict):
            return emit_failure(
                "invalid_manifest",
                f"manifest root must be object: {manifest_path.name}",
            )

        if payload.get("wrapper_name") == wrapper_name:
            matches.append((manifest_path, payload))

    if not matches:
        return emit_failure(
            "unknown_wrapper",
            f"unknown lane wrapper for dispatch: {wrapper_name}",
        )
    if len(matches) > 1:
        matching = ", ".join(path.name for path, _ in matches)
        return emit_failure(
            "duplicate_wrapper",
            f"wrapper {wrapper_name} matches multiple manifests: {matching}",
        )

    manifest_path, payload = matches[0]
    phase = payload.get("phase")
    phases = payload.get("phases")
    if not isinstance(phase, str) or phase.strip() == "":
        return emit_failure(
            "invalid_phase",
            f"manifest {manifest_path.name} missing non-empty phase for wrapper {wrapper_name}",
        )
    if not isinstance(phases, dict) or phase not in phases:
        return emit_failure(
            "invalid_phase",
            f"manifest {manifest_path.name} phase {phase!r} not present in phases map",
        )
    if required_phase and phase != required_phase:
        return emit_failure(
            "required_phase_mismatch",
            f"manifest {manifest_path.name} phase {phase!r} does not match required phase {required_phase!r}",
        )

    print("status=ok")
    print(f"manifest_path={manifest_path.resolve()}")
    print(f"phase={phase}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
