#!/usr/bin/env python3
"""Treasury disbursement contract-lane runner."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
from pathlib import Path

MAX_RUNTIME_SECONDS = 90


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/treasury/run_treasury_disbursement_contract_lane.sh")


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
    """Execute treasury disbursement contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    start_time = time.monotonic()
    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/treasury/check_treasury_disbursement_policy.sh"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "treasury-disbursement-go.json"
        generator_code, generator_output = run_capture(
            [
                "bash",
                str(generator),
                "--output-file",
                str(bundle_file),
                "--disbursement-id",
                "disbursement-contract-2026-02-09",
                "--treasury-account-id",
                "treasury-main-001",
                "--destination-account-id",
                "ops-wallet-001",
                "--asset-symbol",
                "KAMN",
                "--disbursement-amount",
                "250000",
                "--daily-limit-amount",
                "500000",
                "--required-approvals",
                "2",
                "--received-approvals",
                "2",
                "--approval-quorum-hash",
                "sha256:approval-contract-2026-02-09",
                "--policy-window-open",
                "true",
                "--ci-fast-gate",
                "PASS",
            ],
            cwd=root_dir,
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected treasury disbursement contract lane decision to be GO")

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_file)],
            cwd=root_dir,
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected treasury disbursement policy check decision to be GO")

    test_code, _test_output = run_capture(
        ["cargo", "test", "-p", "kamn-core", "--test", "release_gonogo_checklist_docs"],
        cwd=root_dir,
    )
    if test_code != 0:
        return fail("expected cargo test -p kamn-core --test release_gonogo_checklist_docs to pass")

    elapsed_seconds = int(time.monotonic() - start_time)
    if elapsed_seconds > MAX_RUNTIME_SECONDS:
        return fail(
            "treasury disbursement contract lane exceeded runtime budget: "
            f"{elapsed_seconds}s"
        )

    print("treasury disbursement contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
