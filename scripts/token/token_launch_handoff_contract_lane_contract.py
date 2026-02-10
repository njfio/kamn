#!/usr/bin/env python3
"""Token launch handoff contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
from pathlib import Path

MAX_RUNTIME_SECONDS = 90


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/token/run_token_launch_handoff_contract_lane.sh")


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
    """Execute token launch handoff contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    start_time = time.monotonic()
    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/token/generate_token_launch_handoff_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/token/check_token_launch_handoff_policy.sh"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "token-launch-handoff-go.json"
        generator_code, generator_output = run_capture(
            [
                "bash",
                str(generator),
                "--output-file",
                str(bundle_file),
                "--token-symbol",
                "KAMN",
                "--configured-total-supply",
                "1000000000",
                "--expected-total-supply",
                "1000000000",
                "--configured-allocation-sum",
                "1000000000",
                "--expected-allocation-sum",
                "1000000000",
                "--allocation-bucket-count",
                "5",
                "--expected-bucket-count",
                "5",
                "--genesis-hash",
                "sha256:token-launch-handoff-go-2026-02-09",
                "--required-approvals",
                "2",
                "--received-approvals",
                "2",
                "--ci-fast-gate",
                "PASS",
            ],
            cwd=root_dir,
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected token launch handoff contract lane decision to be GO")

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_file)],
            cwd=root_dir,
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected token launch handoff policy check decision to be GO")

    cargo_commands = (
        ["cargo", "test", "-p", "kamn-core", "--test", "token_config"],
        ["cargo", "test", "-p", "kamn-core", "--test", "token_config_docs"],
        ["cargo", "test", "-p", "kamn-core", "--test", "release_gonogo_checklist_docs"],
    )
    for command in cargo_commands:
        exit_code, _output = run_capture(command, cwd=root_dir)
        if exit_code != 0:
            return fail(f"expected {' '.join(command)} to pass")

    elapsed_seconds = int(time.monotonic() - start_time)
    if elapsed_seconds > MAX_RUNTIME_SECONDS:
        return fail(
            "token launch handoff contract lane exceeded runtime budget: "
            f"{elapsed_seconds}s"
        )

    print("token launch handoff contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
