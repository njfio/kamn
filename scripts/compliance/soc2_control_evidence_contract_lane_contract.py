#!/usr/bin/env python3
"""SOC2 control evidence contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/compliance/run_soc2_control_evidence_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str]) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main(argv: list[str]) -> int:
    """Execute SOC2 contract-lane smoke checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/compliance/generate_soc2_control_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/compliance/check_soc2_control_evidence_policy.sh"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "soc2-control-contract.json"
        generator_code, generator_output = run_capture(
            [
                "bash",
                str(generator),
                "--output-file",
                str(bundle_file),
                "--control-id",
                "CC6.1",
                "--audit-period-start",
                "2026-01-01",
                "--audit-period-end",
                "2026-01-31",
                "--collector-did",
                "did:kamn:auditor-contract",
                "--evidence-uri",
                "s3://kamn-audit/soc2/cc6_1/contract/evidence.json",
                "--evidence-sha256",
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--tamper-check",
                "PASS",
                "--completeness-check",
                "PASS",
                "--ci-fast-gate",
                "PASS",
            ]
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected SOC2 contract lane bundle decision to be GO")

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_file)]
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected SOC2 contract lane policy check decision to be GO")
        if "failed_checks=none" not in policy_output:
            return fail("expected SOC2 contract lane to report no failed checks")

    print("soc2 control evidence contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
