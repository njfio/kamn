#!/usr/bin/env python3
"""Channel lifecycle contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/channel/run_channel_lifecycle_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str], *, cwd: Path) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd),
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main(argv: list[str]) -> int:
    """Execute channel lifecycle contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    policy_lane = root_dir / "scripts/channel/run_channel_policy_contract_lane.sh"

    commands = (
        ["cargo", "test", "-p", "kamn-core", "--lib", "channel_models::tests::"],
        ["cargo", "test", "-p", "kamn-core", "--test", "channel_models"],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "docs_contract_wave3_harness",
            "channel_models_docs::",
        ],
        ["bash", str(policy_lane)],
    )
    for command in commands:
        exit_code, _output = run_capture(command, cwd=root_dir)
        if exit_code != 0:
            return fail(f"expected {' '.join(command)} to pass")

    print("channel lifecycle snapshot contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
