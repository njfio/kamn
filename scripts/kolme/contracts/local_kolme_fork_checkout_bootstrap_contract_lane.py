#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork checkout bootstrap checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme fork checkout bootstrap contract lane checks."
    )
    parser.add_argument(
        "--mode",
        default="dry-run",
        choices=("dry-run", "run"),
        help="Emit planned checks or execute local checkout bootstrap checkpoints.",
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-checkout-bootstrap-summary.json",
        help="Checkout bootstrap summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-checkout-bootstrap-policy.json",
        help="Checkout bootstrap policy report output.",
    )
    parser.add_argument(
        "--sync-metadata-report",
        default="/tmp/kolme-local-fork-sync-metadata-summary.json",
        help="Nested sync metadata summary output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="120",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-remote-url",
        default="https://github.com/njfio/kolme_fork.git",
        help="Expected fork remote URL in dry-run mode.",
    )
    parser.add_argument(
        "--expected-remote-url",
        default="https://github.com/njfio/kolme_fork.git",
        help="Expected checkout origin URL.",
    )
    parser.add_argument(
        "--expected-ref",
        default="refs/heads/main",
        help="Expected checkout symbolic head ref.",
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
        "run_local_kolme_fork_checkout_bootstrap_lane.sh",
        "check_local_kolme_fork_checkout_bootstrap_policy.py",
        "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh",
        "Regression: #1663",
    )
    for marker in required_doc_markers:
        if marker not in doc_text:
            raise RuntimeError(f"expected Kolme devnet ops doc marker: {marker}")
    if "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh" not in readme_text:
        raise RuntimeError("expected README to reference checkout bootstrap contract lane")


def create_local_source_repo(root: Path) -> Path:
    source_repo = root / "source_fork"
    source_repo.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "-C", str(source_repo), "init", "-q"], check=True)
    subprocess.run(["git", "-C", str(source_repo), "checkout", "-q", "-b", "main"], check=True)
    subprocess.run(["git", "-C", str(source_repo), "config", "user.email", "ci@example.com"], check=True)
    subprocess.run(["git", "-C", str(source_repo), "config", "user.name", "CI Runner"], check=True)
    (source_repo / "README.md").write_text(
        "checkout bootstrap contract lane fixture\n",
        encoding="utf-8",
    )
    subprocess.run(["git", "-C", str(source_repo), "add", "README.md"], check=True)
    subprocess.run(
        ["git", "-C", str(source_repo), "commit", "-q", "-m", "init checkout bootstrap fixture"],
        check=True,
    )
    return source_repo


def run_lane(args: argparse.Namespace) -> tuple[str, str]:
    expected_reason = "dry_run_no_commands_executed"
    ci_env = dict(os.environ)
    lane_args = [
        "bash",
        str(RUNNER),
        "--mode",
        args.mode,
        "--output-json",
        args.output_json,
        "--sync-metadata-report",
        args.sync_metadata_report,
        "--max-seconds",
        args.max_seconds,
    ]

    if args.mode == "dry-run":
        lane_args.extend(
            [
                "--checkout-path",
                "/tmp/kolme_fork",
                "--fork-remote-url",
                args.fork_remote_url,
                "--expected-remote-url",
                args.expected_remote_url,
                "--expected-ref",
                args.expected_ref,
            ]
        )
    else:
        expected_reason = "fork_checkout_bootstrap_passed"
        ci_env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
        with tempfile.TemporaryDirectory(prefix="kolme-fork-checkout-bootstrap-") as tmpdir:
            temp_root = Path(tmpdir)
            source_repo = create_local_source_repo(temp_root)
            checkout_path = temp_root / "kolme_fork_checkout"
            lane_args.extend(
                [
                    "--checkout-path",
                    str(checkout_path),
                    "--fork-remote-url",
                    str(source_repo),
                    "--expected-remote-url",
                    str(source_repo),
                    "--expected-ref",
                    "refs/heads/main",
                    "--allow-non-default-diagnostic-commands",
                    "--git-version-command",
                    "printf 'git version fixture'",
                    "--cargo-version-command",
                    "printf 'cargo version fixture'",
                    "--rustc-version-command",
                    "printf 'rustc version fixture'",
                ]
            )
            subprocess.run(
                lane_args,
                cwd=ROOT_DIR,
                env=ci_env,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            return expected_reason, args.output_json

    subprocess.run(
        lane_args,
        cwd=ROOT_DIR,
        env=ci_env,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return expected_reason, args.output_json


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    ensure_executable(RUNNER, "checkout bootstrap runner")
    ensure_executable(CHECKER, "checkout bootstrap policy checker")
    ensure_docs()

    start_epoch = time.monotonic()
    expected_reason, report_file = run_lane(args)

    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            report_file,
            "--expected-final-decision",
            "GO",
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            expected_reason,
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > int(args.max_seconds):
        print(
            f"local fork checkout bootstrap contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local fork checkout bootstrap contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
