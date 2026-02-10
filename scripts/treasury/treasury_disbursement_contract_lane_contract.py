#!/usr/bin/env python3
"""Treasury disbursement contract-lane runner."""

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
    print("Usage:\n  bash scripts/treasury/run_treasury_disbursement_contract_lane.sh")


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


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
        run_go_bundle_policy_pair(
            root_dir=root_dir,
            generator=generator,
            generator_args=(
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
            ),
            policy_checker=policy_checker,
            bundle_file=bundle_file,
        )

    test_code, _test_output = run_capture(
        ["cargo", "test", "-p", "kamn-core", "--test", "release_gonogo_checklist_docs"],
        cwd=root_dir,
    )
    if test_code != 0:
        raise ContractLaneError(
            "expected cargo test -p kamn-core --test release_gonogo_checklist_docs to pass"
        )

    enforce_runtime_budget(
        lane_name="treasury disbursement contract lane",
        started_at=start_time,
        max_runtime_seconds=MAX_RUNTIME_SECONDS,
    )

    print("treasury disbursement contract lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractLaneError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
