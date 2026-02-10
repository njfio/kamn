#!/usr/bin/env python3
"""Reputation dispute contract-lane runner."""

from __future__ import annotations

import os
import sys
import tempfile
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_lane_helpers import (  # noqa: E402
    build_default_bundle_args,
    run_capture,
)


def usage() -> None:
    """Print usage text."""
    print(
        "Usage:\n"
        "  bash scripts/reputation/run_reputation_dispute_contract_lane.sh "
        "[--output-file <path>] [--skip-tests]"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def extract_value(output: str, key: str) -> str:
    """Extract key=value line value from command output."""
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def main(argv: list[str]) -> int:
    """Execute reputation dispute contract-lane smoke checks."""
    output_file: str | None = None
    skip_tests = False

    i = 0
    while i < len(argv):
        token = argv[i]
        if token in {"--help", "-h"}:
            usage()
            return 0
        if token == "--output-file":
            if i + 1 >= len(argv):
                return fail("missing value for --output-file")
            output_file = argv[i + 1]
            i += 2
            continue
        if token == "--skip-tests":
            skip_tests = True
            i += 1
            continue
        return fail(f"unknown argument: {token}")

    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/reputation/generate_reputation_dispute_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/reputation/check_reputation_dispute_policy.sh"

    if not generator.is_file():
        return fail("reputation dispute evidence generator is not executable")
    if not policy_checker.is_file():
        return fail("reputation dispute policy checker is not executable")

    if not skip_tests:
        for test_target in (
            "reputation_state_model_docs",
            "reputation_signal_routing_docs",
        ):
            code, _ = run_capture(
                ["cargo", "test", "-p", "kamn-core", "--test", test_target],
                cwd=root_dir,
            )
            if code != 0:
                return fail(
                    "reputation dispute contract lane prerequisite docs tests failed"
                )

    max_seconds_raw = os.environ.get("REPUTATION_DISPUTE_MAX_SECONDS", "90")
    try:
        max_seconds = int(max_seconds_raw)
    except ValueError:
        return fail("REPUTATION_DISPUTE_MAX_SECONDS must be an integer")
    if max_seconds < 1:
        return fail("REPUTATION_DISPUTE_MAX_SECONDS must be >= 1")

    start = time.monotonic()

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_path = (
            Path(output_file)
            if output_file is not None
            else Path(temp_dir) / "reputation-dispute-contract.json"
        )

        generator_args = build_default_bundle_args(
            output_file=str(bundle_path),
            pairs=(
                ("--dispute-id", "dispute-contract-001"),
                ("--subject-did", "did:kamn:agent-contract-001"),
                ("--reviewer-did", "did:kamn:reviewer-contract-001"),
                ("--dispute-reason-code", "QUALITY"),
                (
                    "--evidence-uri",
                    "s3://kamn-audit/reputation/dispute-contract-001.json",
                ),
                (
                    "--evidence-sha256",
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                ),
                ("--evidence-hash-verified", "PASS"),
                ("--original-trust-score", "620"),
                ("--proposed-trust-score", "570"),
                ("--max-adjustment-points", "90"),
                ("--policy-window-open", "true"),
                ("--approval-recorded", "true"),
                ("--ci-fast-gate", "PASS"),
            ),
        )
        generator_code, generator_output = run_capture(
            ["bash", str(generator), *generator_args],
            cwd=root_dir,
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected reputation dispute contract lane bundle decision to be GO")

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_path)],
            cwd=root_dir,
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected reputation dispute contract lane policy decision to be GO")
        if "failed_checks=none" not in policy_output:
            return fail("expected reputation dispute contract lane to report no failed checks")

        elapsed_seconds = int(time.monotonic() - start)
        if elapsed_seconds > max_seconds:
            return fail(
                "reputation dispute contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("status=ok")
        print(f"bundle_file={bundle_path}")
        print(f"reason_key={extract_value(generator_output, 'reason_key')}")
        print(f"final_decision={extract_value(policy_output, 'final_decision')}")
        print("reputation dispute contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
