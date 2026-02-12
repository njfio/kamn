#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork checkout bootstrap checks."""

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
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
PIN_MANIFEST_SCHEMA = "kamn.kolme.fork-pin-manifest.v1"
PIN_MANIFEST_FIXTURE = ROOT_DIR / "fixtures/kolme_compatibility/kolme_fork_pin_manifest.json"


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
    parser.add_argument(
        "--fork-pin-manifest-file",
        default=str(PIN_MANIFEST_FIXTURE),
        help="Fork pin manifest file providing remote/ref/commit tuple.",
    )
    return parser


def ensure_executable(path: Path, description: str) -> None:
    if not path.is_file() or not path.stat().st_mode & 0o111:
        raise RuntimeError(f"expected executable {description}: {path}")


def ensure_docs() -> None:
    if not DOC_FILE.is_file():
        raise RuntimeError("expected Kolme devnet ops documentation to exist")
    if not CI_DOC_FILE.is_file():
        raise RuntimeError("expected CI strategy documentation to exist")
    if not README_FILE.is_file():
        raise RuntimeError("expected README to exist")
    required_doc_markers = (
        "run_local_kolme_fork_checkout_bootstrap_lane.sh",
        "check_local_kolme_fork_checkout_bootstrap_policy.py",
        "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh",
        "--fork-pin-manifest-file",
        "--expected-commit",
        "fork_pin_manifest_schema_version=kamn.kolme.fork-pin-manifest.v1",
        "head_commit_mismatch",
        "Regression: #1663",
        "Regression: #2328",
    )
    for doc_path in (DOC_FILE, CI_DOC_FILE, README_FILE):
        doc_text = doc_path.read_text(encoding="utf-8")
        for marker in required_doc_markers:
            if marker not in doc_text:
                raise RuntimeError(f"expected docs marker in {doc_path}: {marker}")


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


def resolve_head_commit(repo_path: Path) -> str:
    return (
        subprocess.check_output(
            ["git", "-C", str(repo_path), "rev-parse", "HEAD"],
            text=True,
        )
        .strip()
    )


def write_fork_pin_manifest(
    *,
    path: Path,
    fork_remote_url: str,
    expected_remote_url: str,
    expected_ref: str,
    expected_commit: str,
) -> None:
    payload = {
        "schema_version": PIN_MANIFEST_SCHEMA,
        "fork_remote_url": fork_remote_url,
        "expected_remote_url": expected_remote_url,
        "expected_ref": expected_ref,
        "expected_commit": expected_commit,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def run_policy_check(
    *,
    report_file: Path,
    output_json: Path,
    expected_final_decision: str,
    required_reason_code: str,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(report_file),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            required_reason_code,
            "--output-json",
            str(output_json),
        ],
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )


def run_bootstrap_lane(
    *,
    mode: str,
    output_json: Path,
    sync_metadata_report: Path,
    max_seconds: str,
    checkout_path: Path,
    fork_remote_url: str,
    expected_remote_url: str,
    expected_ref: str,
    fork_pin_manifest_file: Path,
    allow_non_default_diagnostics: bool,
) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    if mode == "run":
        env["KAMN_KOLME_LOCAL_HEAVY"] = "1"

    lane_args = [
        "bash",
        str(RUNNER),
        "--mode",
        mode,
        "--output-json",
        str(output_json),
        "--sync-metadata-report",
        str(sync_metadata_report),
        "--max-seconds",
        max_seconds,
        "--checkout-path",
        str(checkout_path),
        "--fork-remote-url",
        fork_remote_url,
        "--expected-remote-url",
        expected_remote_url,
        "--expected-ref",
        expected_ref,
        "--fork-pin-manifest-file",
        str(fork_pin_manifest_file),
    ]
    if allow_non_default_diagnostics:
        lane_args.extend(
            [
                "--allow-non-default-diagnostic-commands",
                "--git-version-command",
                "printf 'git version fixture'",
                "--cargo-version-command",
                "printf 'cargo version fixture'",
                "--rustc-version-command",
                "printf 'rustc version fixture'",
            ]
        )

    return subprocess.run(
        lane_args,
        cwd=ROOT_DIR,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    ensure_executable(RUNNER, "checkout bootstrap runner")
    ensure_executable(CHECKER, "checkout bootstrap policy checker")
    ensure_docs()
    pin_manifest_fixture = Path(args.fork_pin_manifest_file).resolve()
    if not pin_manifest_fixture.is_file():
        print("expected fork pin manifest fixture to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    dry_run_report_file = Path(args.output_json).resolve()
    dry_run_sync_metadata = Path(args.sync_metadata_report).resolve()
    dry_run_result = run_bootstrap_lane(
        mode="dry-run",
        output_json=dry_run_report_file,
        sync_metadata_report=dry_run_sync_metadata,
        max_seconds=args.max_seconds,
        checkout_path=Path("/tmp/kolme_fork"),
        fork_remote_url=args.fork_remote_url,
        expected_remote_url=args.expected_remote_url,
        expected_ref=args.expected_ref,
        fork_pin_manifest_file=pin_manifest_fixture,
        allow_non_default_diagnostics=False,
    )
    if dry_run_result.returncode != 0:
        print("expected dry-run checkout bootstrap lane to pass", file=sys.stderr)
        stderr = dry_run_result.stderr.strip()
        if stderr:
            print(stderr, file=sys.stderr)
        return 1

    dry_run_policy_result = run_policy_check(
        report_file=dry_run_report_file,
        output_json=Path(args.policy_output_json).resolve(),
        expected_final_decision="GO",
        required_reason_code="dry_run_no_commands_executed",
    )
    if dry_run_policy_result.returncode != 0:
        print("expected checkout bootstrap policy checker GO path to pass in dry-run mode", file=sys.stderr)
        stderr = dry_run_policy_result.stderr.strip()
        if stderr:
            print(stderr, file=sys.stderr)
        return 1

    dry_run_summary = json.loads(dry_run_report_file.read_text(encoding="utf-8"))
    if dry_run_summary.get("schema_version") != "kamn.kolme.local-fork-checkout-bootstrap-summary.v1":
        print("unexpected checkout bootstrap dry-run summary schema", file=sys.stderr)
        return 1
    if dry_run_summary.get("commit_pin_enforced") is not True:
        print("expected checkout bootstrap dry-run summary commit_pin_enforced=true", file=sys.stderr)
        return 1
    if dry_run_summary.get("fork_pin_manifest_schema_version") != PIN_MANIFEST_SCHEMA:
        print(
            "expected fork_pin_manifest_schema_version=kamn.kolme.fork-pin-manifest.v1 in dry-run summary",
            file=sys.stderr,
        )
        return 1
    if not isinstance(dry_run_summary.get("expected_commit"), str) or len(
        dry_run_summary.get("expected_commit", "")
    ) != 40:
        print("expected checkout bootstrap dry-run summary expected_commit marker", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="kolme-fork-checkout-bootstrap-contract-") as tmpdir:
        temp_root = Path(tmpdir)
        source_repo = create_local_source_repo(temp_root)
        source_commit = resolve_head_commit(source_repo)
        checkout_path = temp_root / "kolme_fork_checkout"

        matching_manifest = temp_root / "fork-pin-manifest-match.json"
        write_fork_pin_manifest(
            path=matching_manifest,
            fork_remote_url=str(source_repo),
            expected_remote_url=str(source_repo),
            expected_ref="refs/heads/main",
            expected_commit=source_commit,
        )
        run_mode_report = temp_root / "checkout-bootstrap-run-summary.json"
        run_mode_sync = temp_root / "sync-metadata-run-summary.json"
        run_mode_policy = temp_root / "checkout-bootstrap-run-policy.json"
        run_mode_result = run_bootstrap_lane(
            mode="run",
            output_json=run_mode_report,
            sync_metadata_report=run_mode_sync,
            max_seconds=args.max_seconds,
            checkout_path=checkout_path,
            fork_remote_url=str(source_repo),
            expected_remote_url=str(source_repo),
            expected_ref="refs/heads/main",
            fork_pin_manifest_file=matching_manifest,
            allow_non_default_diagnostics=True,
        )
        if run_mode_result.returncode != 0:
            print("expected run-mode checkout bootstrap lane to pass with matching fork pin manifest", file=sys.stderr)
            stderr = run_mode_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1

        run_mode_policy_result = run_policy_check(
            report_file=run_mode_report,
            output_json=run_mode_policy,
            expected_final_decision="GO",
            required_reason_code="fork_checkout_bootstrap_passed",
        )
        if run_mode_policy_result.returncode != 0:
            print("expected checkout bootstrap policy checker GO path to pass in run mode", file=sys.stderr)
            stderr = run_mode_policy_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1

        run_mode_summary = json.loads(run_mode_report.read_text(encoding="utf-8"))
        if run_mode_summary.get("expected_commit") != source_commit:
            print("expected checkout bootstrap run summary expected_commit to match source HEAD", file=sys.stderr)
            return 1

        mismatched_manifest = temp_root / "fork-pin-manifest-mismatch.json"
        write_fork_pin_manifest(
            path=mismatched_manifest,
            fork_remote_url=str(source_repo),
            expected_remote_url=str(source_repo),
            expected_ref="refs/heads/main",
            expected_commit="0000000000000000000000000000000000000000",
        )
        mismatch_report = temp_root / "checkout-bootstrap-mismatch-summary.json"
        mismatch_sync_report = temp_root / "sync-metadata-mismatch-summary.json"
        mismatch_result = run_bootstrap_lane(
            mode="run",
            output_json=mismatch_report,
            sync_metadata_report=mismatch_sync_report,
            max_seconds=args.max_seconds,
            checkout_path=checkout_path,
            fork_remote_url=str(source_repo),
            expected_remote_url=str(source_repo),
            expected_ref="refs/heads/main",
            fork_pin_manifest_file=mismatched_manifest,
            allow_non_default_diagnostics=True,
        )
        if mismatch_result.returncode == 0:
            print("expected fork pin manifest commit mismatch to fail closed", file=sys.stderr)
            return 1
        mismatch_summary = json.loads(mismatch_report.read_text(encoding="utf-8"))
        if mismatch_summary.get("reason_code") != "checkpoint_failed_sync_metadata":
            print("expected checkpoint_failed_sync_metadata for commit mismatch fail-closed path", file=sys.stderr)
            return 1
        checks = mismatch_summary.get("checks")
        if not isinstance(checks, list):
            print("expected checks list in mismatch summary", file=sys.stderr)
            return 1
        sync_check = next(
            (entry for entry in checks if isinstance(entry, dict) and entry.get("id") == "sync_metadata"),
            None,
        )
        if not isinstance(sync_check, dict) or sync_check.get("reason_code") != "head_commit_mismatch":
            print("expected head_commit_mismatch reason marker in mismatch summary sync_metadata check", file=sys.stderr)
            return 1

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
