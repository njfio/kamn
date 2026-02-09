#!/usr/bin/env python3
"""Run deterministic TCP failover/reconnect matrix checks for the Rust SDK."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import time
from dataclasses import dataclass


ROOT_DIR = pathlib.Path(__file__).resolve().parents[2]
DEFAULT_FIXTURE = ROOT_DIR / "fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt"
DEFAULT_OUTPUT = pathlib.Path("/tmp/kamn-tcp-failover-reconnect-report.json")


@dataclass(frozen=True)
class Scenario:
    name: str
    command: list[str]


FAST_SCENARIOS = [
    Scenario(
        "fixture_contract",
        [
            "cargo",
            "test",
            "-p",
            "kamn-sdk",
            "--test",
            "tcp_failover_matrix",
            "unit_failover_reconnect_fixture_is_well_formed",
        ],
    ),
    Scenario(
        "primary_loss_reconnect_catchup",
        [
            "cargo",
            "test",
            "-p",
            "kamn-sdk",
            "--test",
            "tcp_failover_matrix",
            "functional_primary_loss_reconnect_and_catchup_matrix_case",
        ],
    ),
    Scenario(
        "three_process_failover",
        [
            "cargo",
            "test",
            "-p",
            "kamn-sdk",
            "--test",
            "tcp_failover_matrix",
            "integration_three_process_failover_matrix_case",
        ],
    ),
    Scenario(
        "reconnect_drift_regression",
        [
            "cargo",
            "test",
            "-p",
            "kamn-sdk",
            "--test",
            "tcp_failover_matrix",
            "regression_reconnect_drift_signature_fixture_contract",
        ],
    ),
    Scenario(
        "fast_lane_budget",
        [
            "cargo",
            "test",
            "-p",
            "kamn-sdk",
            "--test",
            "tcp_failover_matrix",
            "performance_tcp_failover_reconnect_matrix_fast_lane_budget",
        ],
    ),
]

DEEP_SCENARIO = Scenario(
    "deep_reconnect_stress",
    [
        "cargo",
        "test",
        "-p",
        "kamn-sdk",
        "--test",
        "tcp_failover_matrix",
        "performance_tcp_failover_reconnect_matrix_deep_lane",
        "--",
        "--ignored",
    ],
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run deterministic TCP failover/reconnect matrix checks."
    )
    parser.add_argument(
        "--lane",
        choices=("fast", "deep"),
        default="fast",
        help="Execution lane profile.",
    )
    parser.add_argument(
        "--fixture",
        default=str(DEFAULT_FIXTURE),
        help="Path to reconnect drift signature fixture file.",
    )
    parser.add_argument(
        "--output-json",
        default=str(DEFAULT_OUTPUT),
        help="Path for JSON report output.",
    )
    parser.add_argument(
        "--max-cases",
        type=int,
        default=None,
        help="Optional cap for executed scenario count (for smoke validation).",
    )
    return parser.parse_args()


def read_fixture_cases(path: pathlib.Path) -> list[dict[str, str]]:
    cases: list[dict[str, str]] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "|" not in line:
            raise ValueError(
                f"fixture line must contain scenario|expected_signature: {line}"
            )
        scenario, signature = line.split("|", 1)
        cases.append(
            {
                "scenario": scenario.strip(),
                "expected_signature": signature.strip(),
            }
        )
    return cases


def run_scenario(scenario: Scenario) -> dict[str, object]:
    started = time.monotonic()
    completed = subprocess.run(
        scenario.command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    elapsed_ms = int((time.monotonic() - started) * 1000)
    return {
        "name": scenario.name,
        "status": "pass" if completed.returncode == 0 else "fail",
        "duration_ms": elapsed_ms,
        "return_code": completed.returncode,
        "command": " ".join(scenario.command),
        "stderr_tail": "\n".join(completed.stderr.strip().splitlines()[-8:]),
    }


def main() -> int:
    args = parse_args()
    fixture_path = pathlib.Path(args.fixture).resolve()
    output_json = pathlib.Path(args.output_json)
    if args.max_cases is not None and args.max_cases < 1:
        raise SystemExit("--max-cases must be at least 1")

    fixture_cases = read_fixture_cases(fixture_path)
    scenarios: list[Scenario] = list(FAST_SCENARIOS)
    if args.lane == "deep":
        scenarios.append(DEEP_SCENARIO)
    if args.max_cases is not None:
        scenarios = scenarios[: args.max_cases]

    results = [run_scenario(scenario) for scenario in scenarios]
    failed_count = sum(1 for result in results if result["status"] == "fail")
    status = "pass" if failed_count == 0 else "fail"

    report = {
        "schema_version": "kamn.sdk.tcp-failover-reconnect.matrix.v1",
        "status": status,
        "lane": args.lane,
        "fixture": str(fixture_path.relative_to(ROOT_DIR)),
        "fixture_case_count": len(fixture_cases),
        "scenario_count": len(results),
        "failed_count": failed_count,
        "results": results,
    }

    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(
        f"status={status}; lane={args.lane}; scenarios={len(results)}; failed={failed_count}; output={output_json}"
    )
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
