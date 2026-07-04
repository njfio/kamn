#!/usr/bin/env python3
"""Channel policy contract-lane runner."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/channel/run_channel_policy_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str], *, cwd: Path, env: dict[str, str]) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd),
        env=env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def contract_commands(root_dir: Path) -> tuple[list[str], ...]:
    """Return channel policy commands in stable execution order."""
    retention_lane = root_dir / "scripts/channel/run_channel_retention_redaction_contract_lane.sh"
    return (
        ["cargo", "test", "-p", "kamn-core", "--lib", "channel_policies::tests::"],
        ["cargo", "test", "-p", "kamn-core", "--test", "channel_permissions_retention"],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "docs_contract_wave3_harness",
            "channel_permissions_retention_docs::",
        ],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "docs_contract_wave3_harness",
            "channel_models_and_permissions_docs::",
        ],
        ["bash", str(retention_lane)],
    )


def resolve_channel_target_dir(root_dir: Path) -> Path:
    """Return the isolated target dir for channel policy contract tests."""
    target_override = os.environ.get("KAMN_CHANNEL_POLICY_CONTRACT_TARGET_DIR")
    channel_target_dir = (
        Path(target_override)
        if target_override
        else root_dir / "target/channel-policy-contract"
    )
    channel_target_dir.mkdir(parents=True, exist_ok=True)
    return channel_target_dir


def main(argv: list[str]) -> int:
    """Execute channel policy contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    channel_target_dir = resolve_channel_target_dir(root_dir)
    cargo_env = {**os.environ, "CARGO_TARGET_DIR": str(channel_target_dir)}
    for command in contract_commands(root_dir):
        exit_code, _output = run_capture(command, cwd=root_dir, env=cargo_env)
        if exit_code != 0:
            return fail(f"expected {' '.join(command)} to pass")

    print("channel policy contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
