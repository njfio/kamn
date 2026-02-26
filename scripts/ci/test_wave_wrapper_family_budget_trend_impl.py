#!/usr/bin/env python3
"""Contract tests for wave wrapper-family budget trend checkers."""

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
        description="Validate wave wrapper-family budget trend checker behavior",
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


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parents[2]
    python_checker = root / "scripts/ci/kolme_wrapper_inventory_baseline.py"

    if args.family == "kolme":
        wave_label = f"Kolme wave-{args.wave_id}"
        trend_checker = root / f"scripts/ci/check_kolme_wave{args.wave_id}_wrapper_family_budget_trend.sh"
        threshold_file = root / f"fixtures/ci/kolme_wave{args.wave_id}_wrapper_family_trend_thresholds.json"
        baseline_fixture = root / f"fixtures/ci/kolme_wave{args.wave_id}_wrapper_family_baseline.json"
        matrix_fixture = root / f"fixtures/ci/kolme_wave{args.wave_id}_wrapper_family_matrix.json"
    else:
        wave_label = f"non-Kolme wave-{args.wave_id}"
        trend_checker = root / f"scripts/ci/check_non_kolme_wave{args.wave_id}_wrapper_family_budget_trend.sh"
        threshold_file = root / f"fixtures/ci/non_kolme_wave{args.wave_id}_wrapper_family_trend_thresholds.json"
        baseline_fixture = root / f"fixtures/ci/non_kolme_wave{args.wave_id}_wrapper_family_baseline.json"
        matrix_fixture = root / f"fixtures/ci/non_kolme_wave{args.wave_id}_wrapper_family_matrix.json"

    require_executable(trend_checker, f"expected {wave_label} trend checker wrapper to be executable")
    require_executable(python_checker, "expected python baseline checker script to be executable")
    require_file(threshold_file, f"expected {wave_label} trend threshold fixture to exist")
    require_file(baseline_fixture, f"expected {wave_label} baseline fixture to exist")
    require_file(matrix_fixture, f"expected {wave_label} matrix fixture to exist")

    with tempfile.TemporaryDirectory(prefix="wave-wrapper-trend-") as tmp:
        tmp_dir = Path(tmp)

        pass_report = tmp_dir / "pass-report.json"
        pass_out = tmp_dir / "pass.out"
        run_capture_stdout(
            [
                "bash",
                str(trend_checker),
                "--matrix-file",
                str(matrix_fixture),
                "--baseline-file",
                str(baseline_fixture),
                "--output-json",
                str(pass_report),
            ],
            pass_out,
        )

        assert_file_contains(pass_out, r"^status=pass$", "expected pass status marker")
        assert_file_contains(pass_out, r"^mode=trend$", "expected trend mode marker")
        assert_file_contains(pass_out, r"^wrapper_count_delta=0$", "expected wrapper_count_delta marker")
        assert_file_contains(pass_out, r"^total_shell_loc_delta=0$", "expected total_shell_loc_delta marker")
        assert_file_contains(pass_out, r"^violation_count=0$", "expected violation_count marker")
        assert_file_contains(pass_out, r"^reason_codes=none$", "expected reason_codes marker")

        if args.family == "non_kolme":
            assert_file_contains(
                pass_out,
                r"^reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1$",
                "expected reason taxonomy marker",
            )
            assert_file_contains(pass_out, r"^reason_codes_value=none$", "expected reason_codes_value marker")
            assert_file_contains(pass_out, r"^policy_decision=GO$", "expected policy_decision marker")
            assert_file_contains(pass_out, r"^ci_smoke_budget_status=within$", "expected ci_smoke_budget_status marker")

        mutated_total_baseline = tmp_dir / "mutated-total-baseline.json"
        shutil.copyfile(baseline_fixture, mutated_total_baseline)
        mutate_json(mutated_total_baseline, lambda payload: payload.__setitem__("total_shell_loc", 0))

        fail_total_out = tmp_dir / "fail-total.out"
        run_expect_fail(
            [
                "bash",
                str(trend_checker),
                "--matrix-file",
                str(matrix_fixture),
                "--baseline-file",
                str(mutated_total_baseline),
            ],
            fail_total_out,
            f"expected {wave_label} trend checker to fail when total shell LOC delta exceeds threshold",
        )
        assert_file_contains(
            fail_total_out,
            r"^status=fail$",
            "expected fail status for total baseline mutation",
        )
        assert_file_contains(
            fail_total_out,
            r"^mode=trend$",
            "expected trend mode marker for total baseline mutation",
        )
        assert_file_contains(
            fail_total_out,
            r"total_shell_loc_delta_threshold_exceeded",
            "expected total shell loc threshold marker",
        )
        if args.family == "non_kolme":
            assert_file_contains(
                fail_total_out,
                r"^policy_decision=NO-GO$",
                "expected policy_decision no-go marker",
            )
            assert_file_contains(
                fail_total_out,
                r"^reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1$",
                "expected reason taxonomy marker for total baseline mutation",
            )

        mutated_lane_matrix = tmp_dir / "mutated-lane-matrix.json"
        shutil.copyfile(matrix_fixture, mutated_lane_matrix)
        if args.family == "non_kolme":
            mutated_lane_wrapper = tmp_dir / "run_mutated_contract_lane.sh"
            mutated_lane_wrapper.write_text(
                "#!/usr/bin/env bash\nset -euo pipefail\n\nprintf 'mutated wrapper lane\\n'\n",
                encoding="utf-8",
            )
            mutate_json(
                mutated_lane_matrix,
                lambda payload: payload["lanes"][0].__setitem__(
                    "source_entry",
                    str(mutated_lane_wrapper.resolve()),
                ),
            )
        else:
            alternate_lane_wrapper = root / "scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh"
            require_executable(
                alternate_lane_wrapper,
                f"expected alternate lane wrapper to exist for source-entry drift test: {alternate_lane_wrapper}",
            )
            mutate_json(
                mutated_lane_matrix,
                lambda payload: payload["lanes"][0].__setitem__(
                    "source_entry",
                    str(alternate_lane_wrapper.resolve()),
                ),
            )

        fail_lane_out = tmp_dir / "fail-lane.out"
        run_expect_fail(
            [
                "bash",
                str(trend_checker),
                "--matrix-file",
                str(mutated_lane_matrix),
                "--baseline-file",
                str(baseline_fixture),
            ],
            fail_lane_out,
            f"expected {wave_label} trend checker to fail when lane source_entry drifts from baseline",
        )
        assert_file_contains(
            fail_lane_out,
            r"^status=fail$",
            "expected fail status for lane-source drift mutation",
        )
        assert_file_contains(
            fail_lane_out,
            r"^mode=trend$",
            "expected trend mode marker for lane-source drift mutation",
        )
        if args.family == "non_kolme":
            assert_file_contains(
                fail_lane_out,
                r"lane_source_entry_drift",
                "expected lane_source_entry_drift marker",
            )
        else:
            content = fail_lane_out.read_text(encoding="utf-8", errors="ignore")
            if re.search(r"lane_source_entry_drift|policy_validation_failed", content, re.MULTILINE) is None:
                raise SystemExit("expected lane_source_entry_drift or policy_validation_failed marker")

        mutated_stale_baseline = tmp_dir / "mutated-stale-baseline.json"
        shutil.copyfile(baseline_fixture, mutated_stale_baseline)

        def mutate_lane_id(payload: dict) -> None:
            lanes = payload.get("lanes")
            if not lanes:
                raise SystemExit("expected at least one lane in baseline fixture")
            lane_id = lanes[0].get("lane_id")
            if not isinstance(lane_id, str) or not lane_id.strip():
                raise SystemExit("expected first baseline lane to include a non-empty lane_id")
            lanes[0]["lane_id"] = f"{lane_id}__stale_baseline"

        mutate_json(mutated_stale_baseline, mutate_lane_id)

        fail_stale_out = tmp_dir / "fail-stale.out"
        run_expect_fail(
            [
                "bash",
                str(trend_checker),
                "--matrix-file",
                str(matrix_fixture),
                "--baseline-file",
                str(mutated_stale_baseline),
            ],
            fail_stale_out,
            f"expected {wave_label} trend checker to fail on stale baseline lane inventory",
        )
        assert_file_contains(
            fail_stale_out,
            r"^status=fail$",
            "expected fail status for stale baseline mutation",
        )
        assert_file_contains(
            fail_stale_out,
            r"^mode=trend$",
            "expected trend mode marker for stale baseline mutation",
        )
        assert_file_contains(
            fail_stale_out,
            r"unexpected_new_lanes_in_current_inventory",
            "expected stale baseline marker",
        )

        if args.family == "non_kolme":
            fail_runtime_budget_out = tmp_dir / "fail-runtime-budget.out"
            run_expect_fail(
                [
                    "bash",
                    str(trend_checker),
                    "--matrix-file",
                    str(matrix_fixture),
                    "--baseline-file",
                    str(baseline_fixture),
                    "--max-runtime-seconds",
                    "0",
                ],
                fail_runtime_budget_out,
                f"expected {wave_label} trend checker to fail when CI smoke runtime budget is exceeded",
            )
            assert_file_contains(
                fail_runtime_budget_out,
                r"^status=fail$",
                "expected fail status for runtime budget mutation",
            )
            assert_file_contains(
                fail_runtime_budget_out,
                r"^mode=trend$",
                "expected trend mode marker for runtime budget mutation",
            )
            assert_file_contains(
                fail_runtime_budget_out,
                r"ci_smoke_runtime_budget_exceeded",
                "expected runtime budget marker",
            )
            assert_file_contains(
                fail_runtime_budget_out,
                r"^ci_smoke_budget_status=exceeded$",
                "expected ci_smoke_budget_status exceeded marker",
            )
            assert_file_contains(
                fail_runtime_budget_out,
                r"^policy_decision=NO-GO$",
                "expected policy_decision no-go marker for runtime budget mutation",
            )

    print(f"{wave_label} wrapper-family budget trend checker tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
