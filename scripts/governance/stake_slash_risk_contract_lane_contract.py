#!/usr/bin/env python3
"""Governance stake/slash risk contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/governance/run_stake_slash_risk_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str]) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main(argv: list[str]) -> int:
    """Execute stake/slash risk contract-lane smoke checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/governance/generate_stake_slash_risk_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/governance/check_stake_slash_risk_policy.sh"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "stake-slash-contract.json"
        generator_code, generator_output = run_capture(
            [
                "bash",
                str(generator),
                "--output-file",
                str(bundle_file),
                "--proposal-id",
                "gov-risk-contract-001",
                "--simulation-hash",
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--stake-at-risk-bps",
                "120",
                "--max-stake-at-risk-bps",
                "300",
                "--slash-probability-bps",
                "40",
                "--max-slash-probability-bps",
                "150",
                "--validator-churn-bps",
                "60",
                "--max-validator-churn-bps",
                "180",
                "--quorum-safety-margin-bps",
                "220",
                "--min-quorum-safety-margin-bps",
                "150",
                "--evidence-complete",
                "true",
                "--ci-fast-gate",
                "PASS",
            ]
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected stake/slash contract lane bundle decision to be GO")

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_file)]
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected stake/slash contract lane policy decision to be GO")
        if "failed_checks=none" not in policy_output:
            return fail("expected stake/slash contract lane to report no failed checks")

    print("stake/slash risk contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
