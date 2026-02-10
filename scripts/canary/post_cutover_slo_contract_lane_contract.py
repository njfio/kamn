#!/usr/bin/env python3
"""Post-cutover SLO contract-lane runner."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def usage() -> None:
    """Print usage text."""
    print("Usage:\n  bash scripts/canary/run_post_cutover_slo_contract_lane.sh")


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
    """Execute post-cutover SLO contract-lane checks."""
    if argv and argv[0] in {"--help", "-h"}:
        usage()
        return 0
    if argv:
        return fail(f"unknown argument: {argv[0]}")

    start_time = time.monotonic()
    max_runtime_raw = os.getenv("KAMN_POST_CUTOVER_SLO_MAX_SECONDS", "120")
    if not max_runtime_raw.isdigit():
        return fail("KAMN_POST_CUTOVER_SLO_MAX_SECONDS must be an integer >= 0")
    max_runtime = int(max_runtime_raw)

    root_dir = Path(__file__).resolve().parents[2]
    generator = root_dir / "scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
    policy_checker = root_dir / "scripts/canary/check_post_cutover_slo_policy.sh"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "post-cutover-slo-contract.json"
        generator_code, generator_output = run_capture(
            [
                "bash",
                str(generator),
                "--output-file",
                str(bundle_file),
                "--window-minutes",
                "15",
                "--p95-latency-ms",
                "140",
                "--max-p95-latency-ms",
                "200",
                "--error-rate-bps",
                "18",
                "--max-error-rate-bps",
                "25",
                "--delivery-success-bps",
                "9992",
                "--min-delivery-success-bps",
                "9950",
                "--snapshot-age-seconds",
                "30",
                "--max-snapshot-age-seconds",
                "120",
                "--evidence-complete",
                "true",
                "--ci-fast-gate",
                "PASS",
            ],
            cwd=root_dir,
        )
        if generator_code != 0 or "final_decision=GO" not in generator_output:
            return fail("expected post-cutover SLO contract lane bundle decision to be GO")
        if "reason_key=slo_alert_reason_codes:GO:v1" not in generator_output:
            return fail(
                "expected post-cutover SLO contract lane bundle reason_key to be GO schema marker"
            )

        policy_code, policy_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(bundle_file)],
            cwd=root_dir,
        )
        if policy_code != 0 or "final_decision=GO" not in policy_output:
            return fail("expected post-cutover SLO contract lane policy decision to be GO")
        if "reason_key=slo_alert_reason_codes:GO:v1" not in policy_output:
            return fail(
                "expected post-cutover SLO contract lane policy reason_key to be GO schema marker"
            )

        tampered_bundle = Path(temp_dir) / "post-cutover-slo-alert-drift.json"
        shutil.copyfile(bundle_file, tampered_bundle)
        payload = json.loads(tampered_bundle.read_text(encoding="utf-8"))
        alerts = payload.get("alerts")
        if not isinstance(alerts, dict):
            return fail("expected alerts object in post-cutover SLO evidence bundle")
        alerts["alert_keys"] = ["slo.synthetic.alert.drifted"]
        alerts["total_alerts"] = 1
        alerts["critical_alerts"] = 1
        alerts["warning_alerts"] = 0
        alerts["has_alerts"] = True
        alerts["highest_severity"] = "CRITICAL"
        tampered_bundle.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        tampered_code, tampered_output = run_capture(
            ["bash", str(policy_checker), "--bundle-file", str(tampered_bundle)],
            cwd=root_dir,
        )
        if tampered_code == 0:
            return fail(
                "expected post-cutover SLO contract lane alert drift tamper to fail policy checker"
            )
        if "alerts.alert_keys mismatch" not in tampered_output:
            return fail(
                "expected explicit alert key drift failure from post-cutover SLO policy checker"
            )

    runtime_seconds = int(time.monotonic() - start_time)
    if runtime_seconds > max_runtime:
        return fail(
            "post-cutover SLO contract lane exceeded runtime budget "
            f"({runtime_seconds}s > {max_runtime}s)"
        )

    print("post-cutover SLO contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
