#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork real-process wrapper checks."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
CHECKOUT_BOOTSTRAP_RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh"
CHECKOUT_BOOTSTRAP_CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
PROFILE_PREFLIGHT_RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh"
PROFILE_PREFLIGHT_CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py"
SELF_TEST_RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_self_test_lane.sh"
SELF_TEST_CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_self_test_policy.py"
LIFECYCLE_RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"
LIFECYCLE_CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_real_process_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme fork real-process wrapper contract lane checks."
    )
    parser.add_argument(
        "--mode",
        default="dry-run",
        choices=("dry-run", "run"),
        help="Emit planned checks or execute local-only wrapper checkpoints.",
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-real-process-summary.json",
        help="Real-process wrapper summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-real-process-policy.json",
        help="Real-process wrapper policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="360",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--checkout-path",
        default="/tmp/kolme_fork",
        help="Checkout path used by nested local-fork lanes.",
    )
    parser.add_argument(
        "--fork-remote-url",
        default="https://github.com/njfio/kolme_fork.git",
        help="Fork remote URL used by checkout bootstrap lane.",
    )
    parser.add_argument(
        "--expected-remote-url",
        default="https://github.com/njfio/kolme_fork.git",
        help="Expected origin URL for checkout bootstrap lane.",
    )
    parser.add_argument(
        "--expected-ref",
        default="refs/heads/main",
        help="Expected symbolic head ref for checkout bootstrap lane.",
    )
    parser.add_argument(
        "--base-url",
        default="http://127.0.0.1:3000",
        help="Local Kolme API base URL forwarded to nested checks.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Fork chain version forwarded to nested checks.",
    )
    parser.add_argument(
        "--serve-command",
        default="cargo run --bin example-six-sigma -- serve api-server",
        help="Serve command recorded in wrapper summary.",
    )
    parser.add_argument(
        "--allow-non-fork-serve-command",
        action="store_true",
        help="Allow serve commands that do not target checkout path/cargo run.",
    )
    parser.add_argument(
        "--bootstrap-max-seconds",
        default="120",
        help="Per-check budget for checkout bootstrap lane.",
    )
    parser.add_argument(
        "--preflight-max-seconds",
        default="45",
        help="Per-check budget for profile preflight lane.",
    )
    parser.add_argument(
        "--self-test-max-seconds",
        default="120",
        help="Per-check budget for self-test lane.",
    )
    parser.add_argument(
        "--self-test-matrix-max-seconds",
        default="60",
        help="Per-check budget for nested self-test matrix lane.",
    )
    parser.add_argument(
        "--lifecycle-max-seconds",
        default="300",
        help="Per-check budget for process lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-startup-max-seconds",
        default="45",
        help="Per-check budget for process startup in lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-integration-max-seconds",
        default="240",
        help="Per-check budget for integration verification in lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-bootstrap-max-seconds",
        default="90",
        help="Per-check budget for bootstrap verification in lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-conformance-max-seconds",
        default="180",
        help="Per-check budget for conformance verification in lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-runtime-commit-max-seconds",
        default="30",
        help="Per-check budget for runtime-commit verification in lifecycle lane.",
    )
    parser.add_argument(
        "--lifecycle-mode",
        default="dry-run",
        choices=("dry-run", "run"),
        help="Lifecycle lane execution mode forwarded to nested process lifecycle runner.",
    )
    parser.add_argument(
        "--lifecycle-runtime-commit-finality-command",
        default="",
        help="Optional runtime finality command forwarded to lifecycle lane integration pass-through.",
    )
    parser.add_argument(
        "--lifecycle-runtime-commit-finality-max-seconds",
        default="15",
        help="Per-check budget for runtime finality command in lifecycle lane integration pass-through.",
    )
    parser.add_argument(
        "--lifecycle-runtime-commit-finality-output-file",
        default="/tmp/kolme-local-runtime-commit-live-finality-output.txt",
        help="Output file path for lifecycle lane integration finality command pass-through.",
    )
    parser.add_argument(
        "--self-test-matrix-command",
        action="append",
        default=[],
        help="Optional matrix command forwarded to self-test lane.",
    )
    return parser


def ensure_executable(path: Path, description: str) -> None:
    if not path.is_file() or not path.stat().st_mode & 0o111:
        raise RuntimeError(f"expected executable {description}: {path}")


def ensure_docs() -> None:
    if not DOC_FILE.is_file():
        raise RuntimeError("expected Kolme devnet ops documentation to exist")
    if not README_FILE.is_file():
        raise RuntimeError("expected README to exist")
    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    required_doc_markers = (
        "run_local_kolme_fork_real_process_contract_lane.sh",
        "check_local_kolme_fork_real_process_policy.py",
        "run_local_kolme_fork_checkout_bootstrap_lane.sh",
        "check_local_kolme_fork_checkout_bootstrap_policy.py",
        "Regression: #1644",
        "--lifecycle-mode",
        "--lifecycle-runtime-commit-finality-command",
        "Regression: #1975",
        "Regression: #1977",
    )
    for marker in required_doc_markers:
        if marker not in doc_text:
            raise RuntimeError(f"expected Kolme devnet ops doc marker: {marker}")
    if "run_local_kolme_fork_real_process_contract_lane.sh" not in readme_text:
        raise RuntimeError("expected README to reference real-process wrapper contract lane")
    if "--lifecycle-runtime-commit-finality-command" not in readme_text:
        raise RuntimeError("expected README to document real-process lifecycle finality pass-through option")


def build_contracts() -> dict[str, str]:
    return {
        "default_profile": "example-six-sigma:serve-api-server",
        "expected_cargo_bin": "example-six-sigma",
        "expected_component": "api-server",
        "checkout_bootstrap_runner": "run_local_kolme_fork_checkout_bootstrap_lane.sh",
        "checkout_bootstrap_checker": "check_local_kolme_fork_checkout_bootstrap_policy.py",
        "profile_preflight_runner": "run_local_kolme_fork_profile_preflight_lane.sh",
        "profile_preflight_checker": "check_local_kolme_fork_profile_preflight_policy.py",
        "self_test_runner": "run_local_kolme_fork_self_test_lane.sh",
        "self_test_checker": "check_local_kolme_fork_self_test_policy.py",
        "lifecycle_runner": "run_local_kolme_fork_process_lifecycle_lane.sh",
        "lifecycle_checker": "check_local_kolme_fork_process_lifecycle_policy.py",
    }


def run_command(command: list[str], env: dict[str, str]) -> int:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        env=env,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return result.returncode


def planned_checks(selected_serve_command: str) -> list[dict[str, str]]:
    return [
        {
            "id": "real_fork_command_profile",
            "command": selected_serve_command,
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "checkout_bootstrap_lane",
            "command": "bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode run ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "checkout_bootstrap_policy",
            "command": "python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py --report-file ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "profile_preflight_lane",
            "command": "bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode run ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "profile_preflight_policy",
            "command": "python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "self_test_lane",
            "command": "bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "self_test_policy",
            "command": "python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "process_lifecycle_lane",
            "command": "bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run ...",
            "status": "planned",
            "reason_code": "not_run",
        },
        {
            "id": "process_lifecycle_policy",
            "command": "python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file ...",
            "status": "planned",
            "reason_code": "not_run",
        },
    ]


def run_mode_checks(
    args: argparse.Namespace,
) -> tuple[list[dict[str, str]], str, list[str]]:
    env = dict(os.environ)
    checks: list[dict[str, str]] = []
    observed_reason = "real_fork_process_wrapper_passed"

    with tempfile.TemporaryDirectory(prefix="kolme-fork-real-process-") as temp_dir:
        temp_root = Path(temp_dir)
        source_repo = temp_root / "source_fork"
        checkout_path = temp_root / "kolme_fork"
        bootstrap_report = temp_root / "checkout-bootstrap-summary.json"
        bootstrap_policy = temp_root / "checkout-bootstrap-policy.json"
        preflight_report = temp_root / "profile-preflight-summary.json"
        preflight_policy = temp_root / "profile-preflight-policy.json"
        self_test_report = temp_root / "self-test-summary.json"
        self_test_policy = temp_root / "self-test-policy.json"
        lifecycle_report = temp_root / "lifecycle-summary.json"
        lifecycle_policy = temp_root / "lifecycle-policy.json"
        artifact_paths = [
            str(bootstrap_report),
            str(bootstrap_policy),
            str(preflight_report),
            str(preflight_policy),
            str(self_test_report),
            str(self_test_policy),
            str(lifecycle_report),
            str(lifecycle_policy),
        ]
        if args.lifecycle_runtime_commit_finality_command:
            artifact_paths.append(str(Path(args.lifecycle_runtime_commit_finality_output_file).resolve()))

        source_repo.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "-C", str(source_repo), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(source_repo), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(source_repo), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(source_repo), "config", "user.name", "CI Runner"], check=True)
        (source_repo / "README.md").write_text("real process fixture\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(source_repo), "add", "README.md"], check=True)
        subprocess.run(
            ["git", "-C", str(source_repo), "commit", "-q", "-m", "init real process fixture"],
            check=True,
        )

        if os.environ.get("KAMN_KOLME_LOCAL_HEAVY") != "1":
            checks = planned_checks(args.serve_command)
            for check in checks:
                check["status"] = "fail"
                check["reason_code"] = "local_opt_in_missing"
            return checks, "local_opt_in_missing", artifact_paths

        env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
        checks.append(
            {
                "id": "real_fork_command_profile",
                "command": args.serve_command,
                "status": "pass",
                "reason_code": "serve_command_profile_accepted",
            }
        )

        matrix_args: list[str] = []
        for command in args.self_test_matrix_command:
            matrix_args.extend(["--matrix-command", command])

        process_lifecycle_command = [
            "bash",
            str(LIFECYCLE_RUNNER),
            "--mode",
            args.lifecycle_mode,
            "--checkout-path",
            str(checkout_path),
            "--expected-remote-url",
            str(source_repo),
            "--expected-ref",
            "refs/heads/main",
            "--base-url",
            args.base_url,
            "--fork-chain-version",
            args.fork_chain_version,
            "--max-seconds",
            args.lifecycle_max_seconds,
            "--startup-max-seconds",
            args.lifecycle_startup_max_seconds,
            "--integration-max-seconds",
            args.lifecycle_integration_max_seconds,
            "--integration-bootstrap-max-seconds",
            args.lifecycle_bootstrap_max_seconds,
            "--integration-conformance-max-seconds",
            args.lifecycle_conformance_max_seconds,
            "--integration-runtime-commit-max-seconds",
            args.lifecycle_runtime_commit_max_seconds,
            "--output-json",
            str(lifecycle_report),
        ]
        if args.lifecycle_mode == "run":
            process_lifecycle_command.extend(
                [
                    "--serve-command",
                    args.serve_command,
                ]
            )
        if args.lifecycle_runtime_commit_finality_command:
            process_lifecycle_command.extend(
                [
                    "--integration-runtime-commit-finality-command",
                    args.lifecycle_runtime_commit_finality_command,
                    "--integration-runtime-commit-finality-max-seconds",
                    args.lifecycle_runtime_commit_finality_max_seconds,
                    "--integration-runtime-commit-finality-output-file",
                    args.lifecycle_runtime_commit_finality_output_file,
                ]
            )

        ordered_checks = [
            (
                "checkout_bootstrap_lane",
                [
                    "bash",
                    str(CHECKOUT_BOOTSTRAP_RUNNER),
                    "--mode",
                    "run",
                    "--checkout-path",
                    str(checkout_path),
                    "--fork-remote-url",
                    str(source_repo),
                    "--expected-remote-url",
                    str(source_repo),
                    "--expected-ref",
                    "refs/heads/main",
                    "--sync-metadata-report",
                    str(temp_root / "sync-metadata-summary.json"),
                    "--max-seconds",
                    args.bootstrap_max_seconds,
                    "--allow-non-default-diagnostic-commands",
                    "--git-version-command",
                    "printf 'git version fixture'",
                    "--cargo-version-command",
                    "printf 'cargo version fixture'",
                    "--rustc-version-command",
                    "printf 'rustc version fixture'",
                    "--output-json",
                    str(bootstrap_report),
                ],
                "checkpoint_failed_checkout_bootstrap_lane",
            ),
            (
                "checkout_bootstrap_policy",
                [
                    "python3",
                    str(CHECKOUT_BOOTSTRAP_CHECKER),
                    "--report-file",
                    str(bootstrap_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--require-reason-code",
                    "fork_checkout_bootstrap_passed",
                    "--output-json",
                    str(bootstrap_policy),
                ],
                "checkpoint_failed_checkout_bootstrap_policy",
            ),
            (
                "profile_preflight_lane",
                [
                    "bash",
                    str(PROFILE_PREFLIGHT_RUNNER),
                    "--mode",
                    "dry-run",
                    "--checkout-path",
                    str(checkout_path),
                    "--output-json",
                    str(preflight_report),
                ],
                "checkpoint_failed_profile_preflight_lane",
            ),
            (
                "profile_preflight_policy",
                [
                    "python3",
                    str(PROFILE_PREFLIGHT_CHECKER),
                    "--report-file",
                    str(preflight_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--require-reason-code",
                    "dry_run_no_commands_executed",
                    "--output-json",
                    str(preflight_policy),
                ],
                "checkpoint_failed_profile_preflight_policy",
            ),
            (
                "self_test_lane",
                [
                    "bash",
                    str(SELF_TEST_RUNNER),
                    "--mode",
                    "dry-run",
                    "--checkout-path",
                    str(checkout_path),
                    "--expected-remote-url",
                    str(source_repo),
                    "--expected-ref",
                    "refs/heads/main",
                    "--max-seconds",
                    args.self_test_max_seconds,
                    "--matrix-max-seconds",
                    args.self_test_matrix_max_seconds,
                    "--output-json",
                    str(self_test_report),
                    *matrix_args,
                ],
                "checkpoint_failed_self_test_lane",
            ),
            (
                "self_test_policy",
                [
                    "python3",
                    str(SELF_TEST_CHECKER),
                    "--report-file",
                    str(self_test_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--require-reason-code",
                    "dry_run_no_commands_executed",
                    "--output-json",
                    str(self_test_policy),
                ],
                "checkpoint_failed_self_test_policy",
            ),
            (
                "process_lifecycle_lane",
                process_lifecycle_command,
                "checkpoint_failed_process_lifecycle_lane",
            ),
            (
                "process_lifecycle_policy",
                [
                    "python3",
                    str(LIFECYCLE_CHECKER),
                    "--report-file",
                    str(lifecycle_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--require-reason-code",
                    "dry_run_no_commands_executed",
                    "--output-json",
                    str(lifecycle_policy),
                ],
                "checkpoint_failed_process_lifecycle_policy",
            ),
        ]

        for check_id, command, failure_reason in ordered_checks:
            exit_code = run_command(command, env=env)
            if exit_code == 0:
                checks.append(
                    {
                        "id": check_id,
                        "command": " ".join(command),
                        "status": "pass",
                        "reason_code": f"{check_id}_passed",
                    }
                )
                continue
            checks.append(
                {
                    "id": check_id,
                    "command": " ".join(command),
                    "status": "fail",
                    "reason_code": failure_reason,
                }
            )
            observed_reason = failure_reason
            break

        expected_ids = [
            "checkout_bootstrap_lane",
            "checkout_bootstrap_policy",
            "profile_preflight_lane",
            "profile_preflight_policy",
            "self_test_lane",
            "self_test_policy",
            "process_lifecycle_lane",
            "process_lifecycle_policy",
        ]
        observed_ids = {entry["id"] for entry in checks}
        for check_id in expected_ids:
            if check_id in observed_ids:
                continue
            checks.append(
                {
                    "id": check_id,
                    "command": f"skipped {check_id}",
                    "status": "skipped",
                    "reason_code": "skipped_due_prior_failure",
                }
            )

    return checks, observed_reason, artifact_paths


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    if not args.lifecycle_runtime_commit_finality_max_seconds.isdigit() or int(
        args.lifecycle_runtime_commit_finality_max_seconds
    ) <= 0:
        print("lifecycle-runtime-commit-finality-max-seconds must be a positive integer", file=sys.stderr)
        return 1

    for script, description in (
        (CHECKOUT_BOOTSTRAP_RUNNER, "checkout bootstrap runner"),
        (CHECKOUT_BOOTSTRAP_CHECKER, "checkout bootstrap checker"),
        (PROFILE_PREFLIGHT_RUNNER, "profile preflight runner"),
        (PROFILE_PREFLIGHT_CHECKER, "profile preflight checker"),
        (SELF_TEST_RUNNER, "self-test runner"),
        (SELF_TEST_CHECKER, "self-test checker"),
        (LIFECYCLE_RUNNER, "process lifecycle runner"),
        (LIFECYCLE_CHECKER, "process lifecycle checker"),
        (POLICY_CHECKER, "real-process wrapper policy checker"),
    ):
        ensure_executable(script, description)

    ensure_docs()

    start_epoch = time.monotonic()
    selected_serve_command = args.serve_command
    status = "ok"
    reason_code = "dry_run_no_commands_executed"
    budget_status = "not_run"
    checks = planned_checks(selected_serve_command)
    artifact_paths = [
        "/tmp/kolme-local-fork-checkout-bootstrap-summary.json",
        "/tmp/kolme-local-fork-checkout-bootstrap-policy.json",
        "/tmp/kolme-local-fork-profile-preflight-summary.json",
        "/tmp/kolme-local-fork-profile-preflight-policy.json",
        "/tmp/kolme-local-fork-self-test-summary.json",
        "/tmp/kolme-local-fork-self-test-policy.json",
        "/tmp/kolme-local-fork-process-lifecycle-summary.json",
        "/tmp/kolme-local-fork-process-lifecycle-policy.json",
    ]

    if args.mode == "run":
        budget_status = "within_budget"
        checks, reason_code, artifact_paths = run_mode_checks(args)
        if reason_code != "real_fork_process_wrapper_passed":
            status = "fail"

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > int(args.max_seconds):
        budget_status = "exceeded_budget"
        if status == "ok":
            status = "fail"
            reason_code = "real_fork_process_wrapper_budget_exceeded"

    summary = {
        "schema_version": "kamn.kolme.local-fork-real-process-summary.v1",
        "mode": args.mode,
        "status": status,
        "reason_code": reason_code,
        "local_only_enforced": True,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": int(args.max_seconds),
        "budget_status": budget_status,
        "selected_serve_command": selected_serve_command,
        "allow_non_fork_serve_command": bool(args.allow_non_fork_serve_command),
        "lifecycle_mode": args.lifecycle_mode,
        "lifecycle_runtime_commit_finality_enabled": bool(args.lifecycle_runtime_commit_finality_command),
        "lifecycle_runtime_commit_finality_command": args.lifecycle_runtime_commit_finality_command,
        "lifecycle_runtime_commit_finality_max_seconds": int(args.lifecycle_runtime_commit_finality_max_seconds),
        "lifecycle_runtime_commit_finality_output_file": (
            str(Path(args.lifecycle_runtime_commit_finality_output_file).resolve())
            if args.lifecycle_runtime_commit_finality_command
            else ""
        ),
        "contracts": build_contracts(),
        "checks": checks,
        "artifact_paths": artifact_paths,
    }

    output_path = Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    expected_final_decision = "GO" if status == "ok" else "NO-GO"
    subprocess.run(
        [
            "python3",
            str(POLICY_CHECKER),
            "--report-file",
            str(output_path),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            reason_code,
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    print("local fork real-process wrapper contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
