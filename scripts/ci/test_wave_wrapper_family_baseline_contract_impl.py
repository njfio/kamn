#!/usr/bin/env python3
"""Contract tests for wave wrapper-family baseline generator/checker."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import stat
import subprocess
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate wave wrapper-family baseline checker behavior",
    )
    parser.add_argument("--family", required=True, choices=("kolme", "non_kolme"))
    parser.add_argument("--wave-id", required=True)
    args = parser.parse_args()
    if not args.wave_id.isdigit():
        raise SystemExit(f"wave id must be numeric: {args.wave_id}")
    return args


def require_file(path: Path, message: str) -> None:
    if not path.is_file():
        raise SystemExit(message)


def require_executable(path: Path, message: str) -> None:
    require_file(path, message)
    if not (path.stat().st_mode & stat.S_IXUSR):
        raise SystemExit(message)


def assert_file_contains(path: Path, pattern: str, message: str) -> None:
    content = path.read_text(encoding="utf-8", errors="ignore")
    if re.search(pattern, content, re.MULTILINE) is None:
        raise SystemExit(message)


def run_capture_stdout(cmd: list[str], stdout_file: Path) -> None:
    with stdout_file.open("w", encoding="utf-8") as handle:
        subprocess.run(cmd, check=True, stdout=handle, stderr=subprocess.STDOUT)


def run_expect_fail(cmd: list[str], output_file: Path, failure_message: str) -> None:
    result = subprocess.run(cmd, check=False, capture_output=True, text=True)
    output_file.write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode == 0:
        raise SystemExit(failure_message)


def mutate_json(path: Path, mutator) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    mutator(payload)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def assert_dispatch_registry_entry(
    *,
    registry_path: Path,
    wrapper_rel: str,
    expected_target: str,
    expected_wave_id: str,
) -> None:
    payload = json.loads(registry_path.read_text(encoding="utf-8"))
    entries = payload.get("entries")
    if not isinstance(entries, dict):
        raise SystemExit("expected exec registry entries map")

    entry = entries.get(wrapper_rel)
    if not isinstance(entry, dict):
        raise SystemExit(f"missing exec registry entry for {wrapper_rel}")

    if entry.get("interpreter") != "bash":
        raise SystemExit(f"expected interpreter=bash for {wrapper_rel}")
    if entry.get("target") != expected_target:
        raise SystemExit(
            f"expected target={expected_target} for {wrapper_rel}; found {entry.get('target')!r}"
        )
    if entry.get("passthrough") is not False:
        raise SystemExit(f"expected passthrough=false for {wrapper_rel}")

    expected_args = ["--wave-id", expected_wave_id]
    if entry.get("args_prefix") != expected_args:
        raise SystemExit(
            f"expected args_prefix={expected_args!r} for {wrapper_rel}; found {entry.get('args_prefix')!r}"
        )


def assert_dispatch_wrapper_contract(
    *,
    root: Path,
    registry_path: Path,
    wrapper_rel: str,
    expected_target: str,
    expected_wave_id: str,
) -> None:
    wrapper_path = root / wrapper_rel
    if not wrapper_path.is_symlink():
        raise SystemExit(f"expected wrapper family entrypoint to be symlink-backed: {wrapper_rel}")

    target = wrapper_path.readlink().as_posix()
    if target != "../lib/exec_dispatch.sh":
        raise SystemExit(f"expected {wrapper_rel} to target ../lib/exec_dispatch.sh")

    assert_dispatch_registry_entry(
        registry_path=registry_path,
        wrapper_rel=wrapper_rel,
        expected_target=expected_target,
        expected_wave_id=expected_wave_id,
    )


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parents[2]
    registry_path = root / "scripts/lib/exec_registry.json"

    generate_script = root / "scripts/ci/generate_kolme_wrapper_inventory_baseline.sh"
    check_script = root / "scripts/ci/check_kolme_wrapper_inventory_baseline.sh"
    python_script = root / "scripts/ci/kolme_wrapper_inventory_baseline.py"

    if args.family == "kolme":
        wave_label = f"Kolme wave-{args.wave_id}"
        matrix_fixture = root / f"fixtures/ci/kolme_wave{args.wave_id}_wrapper_family_matrix.json"
        baseline_fixture = root / f"fixtures/ci/kolme_wave{args.wave_id}_wrapper_family_baseline.json"
        missing_wrapper = f"scripts/ci/run_missing_kolme_wave{args.wave_id}_wrapper.sh"
        baseline_dispatch_wrapper_rel = (
            f"scripts/ci/test_kolme_wave{args.wave_id}_wrapper_family_baseline_contract.sh"
        )
        baseline_dispatch_target = "scripts/ci/test_kolme_wave_wrapper_family_baseline_contract_impl.sh"
        trend_dispatch_wrapper_rel = (
            f"scripts/ci/test_check_kolme_wave{args.wave_id}_wrapper_family_budget_trend.sh"
        )
        trend_dispatch_target = "scripts/ci/test_check_kolme_wave_wrapper_family_budget_trend_impl.sh"
    else:
        wave_label = f"non-Kolme wave-{args.wave_id}"
        matrix_fixture = root / f"fixtures/ci/non_kolme_wave{args.wave_id}_wrapper_family_matrix.json"
        baseline_fixture = root / f"fixtures/ci/non_kolme_wave{args.wave_id}_wrapper_family_baseline.json"
        missing_wrapper = f"scripts/ci/run_missing_non_kolme_wave{args.wave_id}_wrapper.sh"
        baseline_dispatch_wrapper_rel = (
            f"scripts/ci/test_non_kolme_wave{args.wave_id}_wrapper_family_baseline_contract.sh"
        )
        baseline_dispatch_target = "scripts/ci/test_non_kolme_wave_wrapper_family_baseline_contract_impl.sh"
        trend_dispatch_wrapper_rel = (
            f"scripts/ci/test_check_non_kolme_wave{args.wave_id}_wrapper_family_budget_trend.sh"
        )
        trend_dispatch_target = "scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh"

    assert_dispatch_wrapper_contract(
        root=root,
        registry_path=registry_path,
        wrapper_rel=baseline_dispatch_wrapper_rel,
        expected_target=baseline_dispatch_target,
        expected_wave_id=args.wave_id,
    )
    assert_dispatch_wrapper_contract(
        root=root,
        registry_path=registry_path,
        wrapper_rel=trend_dispatch_wrapper_rel,
        expected_target=trend_dispatch_target,
        expected_wave_id=args.wave_id,
    )

    require_executable(generate_script, "expected baseline generator wrapper to be executable")
    require_executable(check_script, "expected baseline checker wrapper to be executable")
    require_executable(python_script, "expected baseline policy python script to be executable")
    require_file(matrix_fixture, f"expected {wave_label} wrapper-family matrix fixture to exist")
    require_file(baseline_fixture, f"expected {wave_label} wrapper-family baseline fixture to exist")

    with tempfile.TemporaryDirectory(prefix="wave-wrapper-baseline-") as tmp:
        tmp_dir = Path(tmp)

        baseline_payload = json.loads(baseline_fixture.read_text(encoding="utf-8"))
        expected_wrapper_count = int(baseline_payload["wrapper_count"])
        expected_symlink_wrapper_count = int(baseline_payload["symlink_wrapper_count"])
        expected_regular_file_wrapper_count = int(baseline_payload["regular_file_wrapper_count"])
        expected_total_shell_loc = int(baseline_payload["total_shell_loc"])

        generated_baseline = tmp_dir / "generated-baseline.json"
        generate_out = tmp_dir / "generate.out"
        run_capture_stdout(
            [
                "bash",
                str(generate_script),
                "--matrix-file",
                str(matrix_fixture),
                "--output-json",
                str(generated_baseline),
            ],
            generate_out,
        )

        assert_file_contains(generate_out, r"^status=generated$", "expected generated baseline status marker")
        assert_file_contains(
            generate_out,
            rf"^wrapper_count={expected_wrapper_count}$",
            "expected wrapper count marker",
        )
        assert_file_contains(
            generate_out,
            rf"^total_shell_loc={expected_total_shell_loc}$",
            "expected total shell loc marker",
        )

        generated_payload = json.loads(generated_baseline.read_text(encoding="utf-8"))
        if int(generated_payload["wrapper_count"]) != expected_wrapper_count:
            raise SystemExit(f"expected {wave_label} wrapper_count to be {expected_wrapper_count}")
        if int(generated_payload["symlink_wrapper_count"]) != expected_symlink_wrapper_count:
            raise SystemExit(
                f"expected {wave_label} symlink_wrapper_count to be {expected_symlink_wrapper_count}"
            )
        if int(generated_payload["regular_file_wrapper_count"]) != expected_regular_file_wrapper_count:
            raise SystemExit(
                f"expected {wave_label} regular_file_wrapper_count to be {expected_regular_file_wrapper_count}"
            )
        if int(generated_payload["total_shell_loc"]) != expected_total_shell_loc:
            raise SystemExit(f"expected {wave_label} total_shell_loc to be {expected_total_shell_loc}")

        if baseline_payload != generated_payload:
            raise SystemExit(f"{wave_label} generated baseline fixture drift detected")

        delta_report = tmp_dir / "delta-report.json"
        check_pass_out = tmp_dir / "check-pass.out"
        run_capture_stdout(
            [
                "bash",
                str(check_script),
                "--matrix-file",
                str(matrix_fixture),
                "--baseline-file",
                str(baseline_fixture),
                "--output-json",
                str(delta_report),
            ],
            check_pass_out,
        )

        assert_file_contains(check_pass_out, r"^status=pass$", "expected pass status marker")
        assert_file_contains(check_pass_out, r"^wrapper_count_delta=0$", "expected wrapper_count_delta marker")
        assert_file_contains(check_pass_out, r"^total_shell_loc_delta=0$", "expected total_shell_loc_delta marker")
        assert_file_contains(check_pass_out, r"^violation_count=0$", "expected violation_count marker")
        assert_file_contains(check_pass_out, r"^reason_codes=none$", "expected reason_codes marker")

        mutated_baseline = tmp_dir / "mutated-baseline.json"
        shutil.copyfile(baseline_fixture, mutated_baseline)

        def mutate_baseline(payload: dict) -> None:
            payload["lanes"][0]["shell_loc"] = payload["lanes"][0]["shell_loc"] + 1
            payload["total_shell_loc"] = payload["total_shell_loc"] + 1

        mutate_json(mutated_baseline, mutate_baseline)

        check_mutated_baseline_out = tmp_dir / "check-mutated-baseline.out"
        run_expect_fail(
            [
                "bash",
                str(check_script),
                "--matrix-file",
                str(matrix_fixture),
                "--baseline-file",
                str(mutated_baseline),
            ],
            check_mutated_baseline_out,
            f"expected {wave_label} baseline checker to fail on shell_loc drift",
        )
        assert_file_contains(
            check_mutated_baseline_out,
            r"^status=fail$",
            "expected fail status for mutated baseline",
        )
        assert_file_contains(
            check_mutated_baseline_out,
            r"^reason_codes=lane_shell_loc_drift$",
            "expected lane_shell_loc_drift reason code",
        )
        assert_file_contains(
            check_mutated_baseline_out,
            r"shell_loc drifted",
            "expected shell_loc drift detail",
        )

        mutated_matrix = tmp_dir / "mutated-matrix.json"
        shutil.copyfile(matrix_fixture, mutated_matrix)

        def mutate_matrix(payload: dict) -> None:
            payload["lanes"][0]["source_entry"] = missing_wrapper

        mutate_json(mutated_matrix, mutate_matrix)

        check_mutated_matrix_out = tmp_dir / "check-mutated-matrix.out"
        run_expect_fail(
            [
                "bash",
                str(check_script),
                "--matrix-file",
                str(mutated_matrix),
                "--baseline-file",
                str(baseline_fixture),
            ],
            check_mutated_matrix_out,
            f"expected {wave_label} baseline checker to fail when matrix source_entry wrapper is missing",
        )
        assert_file_contains(
            check_mutated_matrix_out,
            r"lane wrapper path does not exist",
            "expected missing lane wrapper marker",
        )

    print(f"{wave_label} wrapper-family baseline contract tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
