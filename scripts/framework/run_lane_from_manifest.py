#!/usr/bin/env python3
"""Manifest-backed lane runner entrypoint."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

from lane_manifest import load_manifest_file, run_lane_phase


def parse_args(argv: list[str]) -> argparse.Namespace:
    """Parse CLI arguments for manifest lane runner."""
    parser = argparse.ArgumentParser(
        description="Run a configured lane phase from a manifest file."
    )
    parser.add_argument("--manifest", required=True, help="Path to lane manifest JSON file.")
    parser.add_argument("--phase", required=True, help="Phase key to execute from manifest.")
    parser.add_argument(
        "--cwd",
        default="",
        help="Optional working directory for the phase command.",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    """Execute selected manifest phase and emit stable status markers."""
    args = parse_args(argv)
    manifest_path = Path(args.manifest)

    try:
        manifest = load_manifest_file(manifest_path)
        cwd = Path(args.cwd) if args.cwd else None
        code, output = run_lane_phase(manifest, args.phase, cwd=cwd)
    except Exception as exc:  # noqa: BLE001
        print("status=fail")
        print(f"error={exc}")
        return 1

    print(f"lane_id={manifest.lane_id}")
    print(f"phase={args.phase}")
    print(f"exit_code={code}")
    if output.strip() != "":
        print(output, end="" if output.endswith("\n") else "\n")

    if code == 0:
        print("status=ok")
        return 0

    print("status=fail")
    return code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
