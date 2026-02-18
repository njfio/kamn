#!/usr/bin/env python3
"""Contract lane runner for fast-gate native API parity checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
MANIFEST_RUNNER = ROOT_DIR / "scripts/framework/run_manifest_lane.sh"
NONCE_BROADCAST_MANIFEST = (
    ROOT_DIR / "scripts/framework/manifests/kolme_nonce_broadcast_parity_contract_lane.json"
)
NOTIFICATIONS_MANIFEST = (
    ROOT_DIR / "scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json"
)
BLOCK_FALLBACK_MANIFEST = (
    ROOT_DIR / "scripts/framework/manifests/kolme_block_fallback_reconciliation_contract_lane.json"
)
CHECKER = ROOT_DIR / "scripts/kolme/check_fast_gate_native_api_parity_policy.py"
DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
CI_TOOLS_FILE = ROOT_DIR / "scripts/ci/test_ci_tools.sh"


def nonce_broadcast_command() -> list[str]:
    return [
        "bash",
        str(MANIFEST_RUNNER),
        "--manifest",
        str(NONCE_BROADCAST_MANIFEST),
        "--phase",
        "contract",
    ]


def notifications_command() -> list[str]:
    return [
        "bash",
        str(MANIFEST_RUNNER),
        "--manifest",
        str(NOTIFICATIONS_MANIFEST),
        "--phase",
        "contract",
    ]


def block_fallback_command() -> list[str]:
    return [
        "bash",
        str(MANIFEST_RUNNER),
        "--manifest",
        str(BLOCK_FALLBACK_MANIFEST),
        "--phase",
        "contract",
    ]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run fast-gate native API parity contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-fast-gate-native-api-parity-summary.json",
        help="Fast-gate native parity summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-fast-gate-native-api-parity-policy.json",
        help="Fast-gate native parity policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="120",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        choices=("PASS", "FAIL"),
        help="ci-fast-gate policy marker.",
    )
    return parser


def run_check(command: list[str], check_id: str) -> dict[str, str]:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if result.returncode == 0:
        return {
            "id": check_id,
            "command": " ".join(command),
            "status": "pass",
            "reason_code": "passed",
        }
    return {
        "id": check_id,
        "command": " ".join(command),
        "status": "fail",
        "reason_code": f"{check_id}_failed",
    }


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    for script in (MANIFEST_RUNNER, CHECKER):
        if not script.is_file() or not script.stat().st_mode & 0o111:
            print(f"expected executable dependency: {script}", file=sys.stderr)
            return 1
    for manifest in (
        NONCE_BROADCAST_MANIFEST,
        NOTIFICATIONS_MANIFEST,
        BLOCK_FALLBACK_MANIFEST,
    ):
        if not manifest.is_file():
            print(f"expected manifest dependency: {manifest}", file=sys.stderr)
            return 1

    if not DOC_FILE.is_file():
        print("expected CI strategy doc to exist", file=sys.stderr)
        return 1
    if not CI_TOOLS_FILE.is_file():
        print("expected CI tools script to exist", file=sys.stderr)
        return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    if "run_fast_gate_native_api_parity_contract_lane.sh" not in doc_text:
        print("expected CI strategy doc to reference fast-gate native parity lane", file=sys.stderr)
        return 1
    if "check_fast_gate_native_api_parity_policy.py" not in doc_text:
        print("expected CI strategy doc to reference fast-gate native parity policy checker", file=sys.stderr)
        return 1
    if "KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120" not in doc_text:
        print("expected CI strategy doc to include fast-gate parity budget marker", file=sys.stderr)
        return 1

    ci_tools_text = CI_TOOLS_FILE.read_text(encoding="utf-8")
    if "test_run_fast_gate_native_api_parity_contract_lane.sh" not in ci_tools_text:
        print("expected ci-tools regression lane to include fast-gate parity test", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    checks: list[dict[str, str]] = []
    status = "ok"
    reason_code = "fast_gate_native_api_parity_passed"
    budget_status = "within_budget"

    if args.ci_fast_gate == "FAIL":
        status = "fail"
        reason_code = "ci_fast_gate_failed"
        budget_status = "not_run"
        checks = [
            {
                "id": "nonce_broadcast_contract",
                "command": " ".join(nonce_broadcast_command()),
                "status": "fail",
                "reason_code": "ci_fast_gate_failed",
            },
            {
                "id": "notifications_consumer_contract",
                "command": " ".join(notifications_command()),
                "status": "fail",
                "reason_code": "ci_fast_gate_failed",
            },
            {
                "id": "block_fallback_contract",
                "command": " ".join(block_fallback_command()),
                "status": "fail",
                "reason_code": "ci_fast_gate_failed",
            },
        ]
    else:
        checks.append(run_check(nonce_broadcast_command(), "nonce_broadcast_contract"))
        if checks[-1]["status"] == "fail":
            status = "fail"
            reason_code = "nonce_broadcast_contract_failed"
            checks.append(
                {
                    "id": "notifications_consumer_contract",
                    "command": " ".join(notifications_command()),
                    "status": "fail",
                    "reason_code": "skipped_due_prior_failure",
                }
            )
            checks.append(
                {
                    "id": "block_fallback_contract",
                    "command": " ".join(block_fallback_command()),
                    "status": "fail",
                    "reason_code": "skipped_due_prior_failure",
                }
            )
        else:
            checks.append(run_check(notifications_command(), "notifications_consumer_contract"))
            if checks[-1]["status"] == "fail":
                status = "fail"
                reason_code = "notifications_consumer_contract_failed"
                checks.append(
                    {
                        "id": "block_fallback_contract",
                        "command": " ".join(block_fallback_command()),
                        "status": "fail",
                        "reason_code": "skipped_due_prior_failure",
                    }
                )
            else:
                checks.append(run_check(block_fallback_command(), "block_fallback_contract"))
                if checks[-1]["status"] == "fail":
                    status = "fail"
                    reason_code = "block_fallback_contract_failed"

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        budget_status = "exceeded_budget"
        if status == "ok":
            status = "fail"
            reason_code = "native_parity_budget_exceeded"

    summary = {
        "schema_version": "kamn.kolme.fast-gate-native-api-parity-summary.v1",
        "status": status,
        "reason_code": reason_code,
        "ci_fast_gate": args.ci_fast_gate,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "budget_status": budget_status,
        "checks": checks,
    }

    output_path = Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    expected_final_decision = "GO" if status == "ok" else "NO-GO"
    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(output_path),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            args.ci_fast_gate,
            "--require-reason-code",
            reason_code,
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    print("fast-gate native API parity contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
