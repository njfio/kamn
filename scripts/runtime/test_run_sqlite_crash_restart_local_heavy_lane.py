#!/usr/bin/env python3
"""Regression test for sqlite crash-restart local-heavy lane wrapper migration."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT_DIR = Path(__file__).resolve().parent.parent.parent
RUNNER = ROOT_DIR / "scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh"
DISPATCHER = ROOT_DIR / "scripts/lib/exec_dispatch.sh"
REGISTRY = ROOT_DIR / "scripts/lib/exec_registry.json"


def fail(message: str) -> None:
    raise SystemExit(message)


def assert_file(path: Path, *, executable: bool = False) -> None:
    if not path.exists():
        fail(f"expected path to exist: {path}")
    if executable and not path.is_file() and not path.is_symlink():
        fail(f"expected executable file/symlink: {path}")


def run_runner(profile: str, output_json: Path) -> str:
    completed = subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--profile",
            profile,
            "--mode",
            "dry-run",
            "--ci-fast-gate",
            "PASS",
            "--max-seconds",
            "240",
            "--output-json",
            str(output_json),
        ],
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"sqlite crash-restart runner failed unexpectedly: {detail}")
    return completed.stdout


def assert_marker(output: str, marker: str) -> None:
    lines = {line.strip() for line in output.splitlines() if line.strip()}
    if marker not in lines:
        fail(f"expected sqlite crash-restart marker: {marker}")


def main() -> int:
    assert_file(RUNNER, executable=True)
    assert_file(DISPATCHER, executable=True)
    assert_file(REGISTRY)

    if not RUNNER.is_symlink():
        fail("expected sqlite crash-restart runner wrapper to be an exec-dispatch symlink")
    if RUNNER.resolve() != DISPATCHER.resolve():
        fail("expected sqlite crash-restart runner wrapper to resolve to exec dispatcher")

    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    entry = registry.get("entries", {}).get(
        "scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh"
    )
    if not isinstance(entry, dict):
        fail("expected registry entry for sqlite crash-restart runner wrapper")
    if entry.get("interpreter") != "python3":
        fail("expected python3 interpreter for sqlite crash-restart runner wrapper")
    if (
        entry.get("target")
        != "scripts/runtime/sqlite_crash_restart_local_heavy_lane_contract.py"
    ):
        fail("expected sqlite crash-restart runner wrapper target in exec registry")
    if entry.get("args_prefix") != []:
        fail("expected empty args_prefix for sqlite crash-restart runner wrapper")
    if entry.get("passthrough") is not True:
        fail("expected passthrough=true for sqlite crash-restart runner wrapper")

    with tempfile.TemporaryDirectory(prefix="sqlite-crash-restart-runner-test-") as tmp_dir:
        tmp = Path(tmp_dir)

        combined_report = tmp / "sqlite-crash-restart-combined.json"
        combined_output = run_runner("combined", combined_report)
        for marker in (
            "status=pass",
            "final_decision=GO",
            "lane_mode=dry-run",
            "profile=combined",
            "profile_status=verified",
            "reason_code=none",
            "restart_drill_status=verified",
            "corruption_drill_status=verified",
            "schema_version=kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1",
            "artifact_schema_version=kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1",
            "reason_taxonomy_version=kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1",
            "reason_codes_csv=crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch",
            "source_report_schema_version=kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1",
            "source_command_count=0",
        ):
            assert_marker(combined_output, marker)

        combined_payload = json.loads(combined_report.read_text(encoding="utf-8"))
        if (
            combined_payload.get("schema_version")
            != "kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1"
        ):
            fail("unexpected sqlite crash-restart lane report schema")
        if (
            combined_payload.get("artifact_schema_version")
            != "kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1"
        ):
            fail("unexpected sqlite crash-restart lane artifact schema marker")
        if (
            combined_payload.get("reason_taxonomy_version")
            != "kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1"
        ):
            fail("unexpected sqlite crash-restart lane reason taxonomy marker")
        if (
            combined_payload.get("reason_codes_csv")
            != "crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch"
        ):
            fail("unexpected sqlite crash-restart lane reason codes csv marker")
        if combined_payload.get("status") != "pass":
            fail("expected sqlite crash-restart lane status=pass")
        if combined_payload.get("final_decision") != "GO":
            fail("expected sqlite crash-restart lane final_decision=GO")
        if combined_payload.get("profile") != "combined":
            fail("expected sqlite crash-restart lane profile=combined")
        if combined_payload.get("profile_status") != "verified":
            fail("expected sqlite crash-restart lane profile_status=verified")
        if combined_payload.get("restart_drill_status") != "verified":
            fail("expected sqlite crash-restart lane restart_drill_status=verified")
        if combined_payload.get("corruption_drill_status") != "verified":
            fail("expected sqlite crash-restart lane corruption_drill_status=verified")
        if (
            combined_payload.get("source_report_schema_version")
            != "kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1"
        ):
            fail("expected sqlite crash-restart lane source report schema marker")
        if combined_payload.get("source_command_count") != 0:
            fail("expected sqlite crash-restart lane source_command_count=0 on dry-run")

        for profile in ("restart", "corruption"):
            report_file = tmp / f"sqlite-crash-restart-{profile}.json"
            output = run_runner(profile, report_file)
            assert_marker(output, f"profile={profile}")

            payload = json.loads(report_file.read_text(encoding="utf-8"))
            if profile == "restart":
                if payload.get("restart_drill_status") != "verified":
                    fail("expected restart profile restart_drill_status=verified")
                if payload.get("corruption_drill_status") != "not_applicable":
                    fail("expected restart profile corruption_drill_status=not_applicable")
            else:
                if payload.get("restart_drill_status") != "not_applicable":
                    fail("expected corruption profile restart_drill_status=not_applicable")
                if payload.get("corruption_drill_status") != "verified":
                    fail("expected corruption profile corruption_drill_status=verified")
            if payload.get("profile_status") != "verified":
                fail("expected profile_status=verified")
            if payload.get("reason_code") != "none":
                fail("expected reason_code=none")

    print("sqlite crash-restart local-heavy lane runner tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
