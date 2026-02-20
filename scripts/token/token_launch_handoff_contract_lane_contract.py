#!/usr/bin/env python3
"""Token launch handoff contract-lane runner."""

from __future__ import annotations

import sys
import tempfile
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_lane_helpers import (  # noqa: E402
    ContractLaneError,
    enforce_runtime_budget,
    run_capture,
    run_go_bundle_policy_pair,
)

MAX_RUNTIME_SECONDS = 90


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/token/run_token_launch_handoff_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


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
        run_go_bundle_policy_pair(
            root_dir=root_dir,
            generator=generator,
            generator_args=(
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
            ),
            policy_checker=policy_checker,
            bundle_file=bundle_file,
        )

    cargo_commands = (
        ["cargo", "test", "-p", "kamn-core", "--test", "token_config"],
        [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "docs_contract_wave3_harness",
            "token_config_docs::",
        ],
        ["cargo", "test", "-p", "kamn-core", "--test", "release_gonogo_checklist_docs"],
    )
    for command in cargo_commands:
        exit_code, _output = run_capture(command, cwd=root_dir)
        if exit_code != 0:
            raise ContractLaneError(f"expected {' '.join(command)} to pass")

    enforce_runtime_budget(
        lane_name="token launch handoff contract lane",
        started_at=start_time,
        max_runtime_seconds=MAX_RUNTIME_SECONDS,
    )

    print("token launch handoff contract lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractLaneError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
