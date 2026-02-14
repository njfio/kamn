#!/usr/bin/env python3
"""Live validation environment lane contract runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, require_non_negative_int, write_json  # noqa: E402

KOLME_LOCAL_HEAVY_ENV = "KAMN_KOLME_LOCAL_HEAVY"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )


def _ensure_executable(path: Path, label: str) -> None:
    if not path.is_file() or not os.access(path, os.X_OK):
        fail(f"expected {label} to be executable")


def run_live_validation_environment_lane(args: argparse.Namespace) -> int:
    mode = args.mode.strip()
    if mode not in {"dry-run", "run"}:
        fail("--mode must be dry-run or run")

    max_seconds = require_non_negative_int("KAMN_LIVE_VALIDATION_ENV_MAX_SECONDS", args.max_seconds)
    topology_max_seconds = require_non_negative_int(
        "KAMN_LIVE_VALIDATION_ENV_TOPOLOGY_MAX_SECONDS",
        args.topology_max_seconds,
    )
    kolme_max_seconds = require_non_negative_int(
        "KAMN_LIVE_VALIDATION_ENV_KOLME_MAX_SECONDS",
        args.kolme_max_seconds,
    )

    if mode == "run" and os.environ.get(KOLME_LOCAL_HEAVY_ENV) != "1":
        fail("run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1")

    topology_runner = ROOT_DIR / "scripts/deploy/validate_deployment_assets_live.sh"
    kolme_bundle_runner = ROOT_DIR / "scripts/kolme/run_local_live_node_validation_bundle_lane.sh"

    _ensure_executable(topology_runner, "deployment assets live validation runner")
    _ensure_executable(kolme_bundle_runner, "local live-node validation bundle runner")

    start_epoch = int(time.time())

    topology_run = _run_command(
        [
            "bash",
            str(topology_runner),
            "--max-seconds",
            str(topology_max_seconds),
        ]
    )
    if topology_run.returncode != 0:
        detail = (topology_run.stderr or topology_run.stdout or "topology command failed").strip()
        fail(f"multi-process topology check failed: {detail}")
    if _extract_line_value(topology_run.stdout, "status") != "pass":
        fail("multi-process topology check did not emit status=pass")
    if _extract_line_value(topology_run.stdout, "asset_contract_status") != "verified":
        fail("multi-process topology check did not emit asset_contract_status=verified")

    kolme_bundle_command = [
        "bash",
        str(kolme_bundle_runner),
        "--mode",
        mode,
        "--checkout-path",
        args.checkout_path,
        "--expected-remote-url",
        args.expected_remote_url,
        "--expected-ref",
        args.expected_ref,
        "--base-url",
        args.base_url,
        "--fork-chain-version",
        args.fork_chain_version,
        "--max-seconds",
        str(kolme_max_seconds),
        "--integration-max-seconds",
        str(min(kolme_max_seconds, 240)),
        "--process-lifecycle-max-seconds",
        str(min(kolme_max_seconds, 300)),
    ]
    kolme_run = _run_command(kolme_bundle_command)
    if kolme_run.returncode != 0:
        detail = (kolme_run.stderr or kolme_run.stdout or "kolme bundle command failed").strip()
        fail(f"kolme connectivity bundle failed: {detail}")
    if _extract_line_value(kolme_run.stdout, "status") != "ok":
        fail("kolme connectivity bundle did not emit status=ok")
    if _extract_line_value(kolme_run.stdout, "local_only_enforced") != "true":
        fail("kolme connectivity bundle did not emit local_only_enforced=true")

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "live validation environment lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": "kamn.runtime.live-validation-environment-report.v1",
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "topology_contract_status": "verified",
        "kolme_connectivity_contract_status": "verified",
        "fail_closed_status": "verified",
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "commands": [
            "scripts/deploy/validate_deployment_assets_live.sh",
            "scripts/kolme/run_local_live_node_validation_bundle_lane.sh",
        ],
    }

    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print("topology_contract_status=verified")
    print("kolme_connectivity_contract_status=verified")
    print("fail_closed_status=verified")
    print(f"elapsed_seconds={elapsed_seconds}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    print("live validation environment lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the live validation environment lane.")
    parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LIVE_VALIDATION_ENV_MODE", "dry-run"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LIVE_VALIDATION_ENV_MAX_SECONDS", "360"),
    )
    parser.add_argument(
        "--topology-max-seconds",
        default=os.environ.get("KAMN_LIVE_VALIDATION_ENV_TOPOLOGY_MAX_SECONDS", "180"),
    )
    parser.add_argument(
        "--kolme-max-seconds",
        default=os.environ.get("KAMN_LIVE_VALIDATION_ENV_KOLME_MAX_SECONDS", "360"),
    )
    parser.add_argument("--checkout-path", default="/tmp/kolme_fork")
    parser.add_argument("--expected-remote-url", default="https://github.com/njfio/kolme_fork.git")
    parser.add_argument("--expected-ref", default="refs/heads/main")
    parser.add_argument("--base-url", default="http://127.0.0.1:3000")
    parser.add_argument("--fork-chain-version", default="v0.15.2")
    parser.set_defaults(handler=run_live_validation_environment_lane)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
