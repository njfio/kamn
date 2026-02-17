#!/usr/bin/env python3
"""Contract lane runner for continuous runtime-commit guardrail checks."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]


def parse_args(argv: list[str]) -> tuple[str, int]:
    output_json = ""
    max_seconds_raw = "180"

    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-json":
            if index + 1 >= len(argv):
                print("missing value for --output-json", file=sys.stderr)
                raise SystemExit(1)
            output_json = argv[index + 1]
            index += 2
            continue
        if argument == "--max-seconds":
            if index + 1 >= len(argv):
                print("missing value for --max-seconds", file=sys.stderr)
                raise SystemExit(1)
            max_seconds_raw = argv[index + 1]
            index += 2
            continue

        print(f"unknown argument: {argument}", file=sys.stderr)
        raise SystemExit(1)

    if not max_seconds_raw.isdigit():
        print("max-seconds must be an integer", file=sys.stderr)
        raise SystemExit(1)

    max_seconds = int(max_seconds_raw)
    if max_seconds <= 0:
        print("max-seconds must be greater than zero", file=sys.stderr)
        raise SystemExit(1)

    return output_json, max_seconds


def main() -> int:
    output_json, max_seconds = parse_args(sys.argv[1:])

    start_epoch = time.monotonic()
    command = [
        "cargo",
        "test",
        "-p",
        "kamn-node",
        "--",
        "rejects_kolme_live_continuous_mode_without_tick_interval",
        "rejects_kolme_live_continuous_mode_without_max_ticks",
        "functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles",
    ]
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )

    test_output = (result.stdout or "") + (result.stderr or "")
    if result.returncode != 0:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print("continuous runtime commit contract lane failed", file=sys.stderr)
        return 1

    required_test_markers = (
        "rejects_kolme_live_continuous_mode_without_tick_interval",
        "rejects_kolme_live_continuous_mode_without_max_ticks",
        "functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles",
    )
    missing_test_markers = [marker for marker in required_test_markers if marker not in test_output]
    if missing_test_markers:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            "expected continuous runtime contract test markers: "
            + ",".join(missing_test_markers),
            file=sys.stderr,
        )
        return 1

    pass_count_match = re.search(r"test result: ok\. (\d+) passed; 0 failed;", test_output)
    if pass_count_match is None:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            "expected continuous runtime contract pass-count marker",
            file=sys.stderr,
        )
        return 1
    if int(pass_count_match.group(1)) < 3:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            "expected at least three continuous runtime contract tests to pass",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            "continuous runtime commit contract lane exceeded runtime budget: "
            f"{elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    payload = {
        "schema_version": "kamn.kolme.continuous-runtime-commit.contract.v1",
        "status": "pass",
        "final_decision": "GO",
        "continuous_mode_status": "verified",
        "finality_recovery_status": "verified",
        "fail_closed_guard_status": "verified",
        "elapsed_seconds": elapsed_seconds,
    }

    if output_json:
        output_path = Path(output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(payload, indent=2) + "\n",
            encoding="utf-8",
        )

    print("status=pass")
    print("final_decision=GO")
    print("continuous_mode_status=verified")
    print("finality_recovery_status=verified")
    print("fail_closed_guard_status=verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
