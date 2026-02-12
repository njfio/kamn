#!/usr/bin/env python3
"""Contract lane runner for local Kolme live deployment preflight checks."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
MAX_SECONDS_ENV = "KAMN_KOLME_LIVE_DEPLOYMENT_PREFLIGHT_CONTRACT_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 12


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme live deployment preflight contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-live-deployment-preflight-summary.json",
        help="Deployment preflight summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-live-deployment-preflight-policy.json",
        help="Deployment preflight policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        type=int,
        default=None,
        help="Runtime budget value passed through summary metadata.",
    )
    return parser


def parse_max_seconds(raw_value: str) -> int:
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError("max-seconds must be a positive integer")
    return int(raw_value)


def run_policy_check(
    report_file: Path,
    output_json: Path,
    expected_final_decision: str,
    required_reason_code: str | None = None,
) -> subprocess.CompletedProcess[str]:
    if required_reason_code is None:
        required_reason_code = (
            "dry_run_no_commands_executed"
            if expected_final_decision == "GO"
            else "checkpoint_failed_runtime_mode_contract"
        )
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


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


def main() -> int:
    args = build_parser().parse_args()

    try:
        max_seconds = (
            args.max_seconds
            if args.max_seconds is not None
            else parse_max_seconds(os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip())
        )
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if max_seconds <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local Kolme live deployment preflight lane runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local Kolme live deployment preflight policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not CI_DOC_FILE.is_file():
        print("expected CI strategy documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "dry-run",
            "--max-seconds",
            str(max_seconds),
            "--output-json",
            args.output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    go_result = run_policy_check(
        report_file=Path(args.output_json),
        output_json=Path(args.policy_output_json),
        expected_final_decision="GO",
    )
    if go_result.returncode != 0:
        print("expected deployment preflight policy checker GO path to pass", file=sys.stderr)
        stderr = go_result.stderr.strip()
        if stderr:
            print(stderr, file=sys.stderr)
        return 1

    summary = json.loads(Path(args.output_json).read_text(encoding="utf-8"))
    policy = json.loads(Path(args.policy_output_json).read_text(encoding="utf-8"))
    if summary.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
        print("unexpected deployment preflight contract-lane summary schema", file=sys.stderr)
        return 1
    if summary.get("status") != "ok":
        print("expected deployment preflight contract-lane summary status ok", file=sys.stderr)
        return 1
    if summary.get("reason_code") != "dry_run_no_commands_executed":
        print("expected deployment preflight dry-run reason code in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("ci_fast_gate_eligible") is not True:
        print("expected deployment preflight contract-lane summary ci_fast_gate_eligible=true", file=sys.stderr)
        return 1
    if summary.get("fallback_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
        print("expected deployment preflight summary fallback signer env marker", file=sys.stderr)
        return 1
    if summary.get("fallback_signer_secret_present") is not False:
        print("expected deployment preflight summary fallback signer secret marker to remain false", file=sys.stderr)
        return 1
    contracts = summary.get("contracts", {})
    if not isinstance(contracts, dict):
        print("expected contracts object in deployment preflight summary", file=sys.stderr)
        return 1
    if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
        print("expected deployment preflight contracts ci_fast_gate_scope=ci-fast-gate", file=sys.stderr)
        return 1
    if contracts.get("fallback_private_key_path_allowed") is not False:
        print("expected deployment preflight contracts to prohibit fallback private key paths", file=sys.stderr)
        return 1
    if contracts.get("approval_quorum_required") != 2:
        print("expected deployment preflight contracts approval_quorum_required=2", file=sys.stderr)
        return 1
    if contracts.get("custody_evidence_required") is not True:
        print("expected deployment preflight contracts custody_evidence_required=true", file=sys.stderr)
        return 1
    if policy.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-policy-report.v1":
        print("unexpected deployment preflight contract-lane policy schema", file=sys.stderr)
        return 1
    if policy.get("final_decision") != "GO":
        print("expected deployment preflight contract-lane policy final_decision GO", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="kolme-deployment-preflight-negative-") as temp_dir:
        temp_path = Path(temp_dir)
        negative_report = temp_path / "runtime_mode_negative_summary.json"
        negative_policy = temp_path / "runtime_mode_negative_policy.json"
        negative_summary = dict(summary)
        negative_summary["runtime_mode"] = "kolme-standard"
        negative_summary["status"] = "fail"
        negative_summary["reason_code"] = "checkpoint_failed_runtime_mode_contract"
        negative_summary["fallback_signer_secret_present"] = True
        negative_report.write_text(json.dumps(negative_summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")

        no_go_result = run_policy_check(
            report_file=negative_report,
            output_json=negative_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_runtime_mode_contract",
        )
        if no_go_result.returncode == 0:
            print("expected deployment preflight runtime-mode negative proof to fail closed", file=sys.stderr)
            return 1
        no_go_policy = json.loads(negative_policy.read_text(encoding="utf-8"))
        no_go_reason_codes = no_go_policy.get("reason_codes")
        if not isinstance(no_go_reason_codes, list):
            print("expected reason_codes list in deployment preflight negative policy output", file=sys.stderr)
            return 1
        if "runtime_mode_mismatch" not in no_go_reason_codes:
            print("expected runtime_mode_mismatch in deployment preflight negative policy output", file=sys.stderr)
            return 1
        if "fallback_signer_secret_present_violation" not in no_go_reason_codes:
            print("expected fallback signer secret presence violation in deployment preflight negative policy output", file=sys.stderr)
            return 1
        if no_go_policy.get("final_decision") != "NO-GO":
            print("expected deployment preflight negative policy final_decision NO-GO", file=sys.stderr)
            return 1

        quorum_negative_report = temp_path / "signer_quorum_negative_summary.json"
        quorum_negative_policy = temp_path / "signer_quorum_negative_policy.json"
        quorum_negative_summary = dict(summary)
        quorum_negative_summary["mode"] = "run"
        quorum_negative_summary["status"] = "fail"
        quorum_negative_summary["reason_code"] = "checkpoint_failed_signer_quorum_contract"
        quorum_negative_summary["signer_secret_present"] = True
        quorum_negative_summary["signer_secret_hex_valid"] = True
        quorum_negative_summary["required_approvals"] = 2
        quorum_negative_summary["received_approvals"] = 1
        quorum_negative_summary["custody_evidence_file"] = ""
        quorum_negative_summary["custody_evidence_present"] = False
        quorum_negative_summary["custody_evidence_sha256"] = ""
        quorum_negative_summary["custody_evidence_sha256_valid"] = False
        quorum_negative_report.write_text(
            json.dumps(quorum_negative_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        quorum_no_go_result = run_policy_check(
            report_file=quorum_negative_report,
            output_json=quorum_negative_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_signer_quorum_contract",
        )
        if quorum_no_go_result.returncode == 0:
            print("expected signer quorum negative proof to fail closed", file=sys.stderr)
            return 1
        quorum_no_go_policy = json.loads(quorum_negative_policy.read_text(encoding="utf-8"))
        quorum_no_go_reason_codes = quorum_no_go_policy.get("reason_codes")
        if not isinstance(quorum_no_go_reason_codes, list):
            print("expected reason_codes list in signer quorum negative policy output", file=sys.stderr)
            return 1
        if "signer_quorum_shortfall" not in quorum_no_go_reason_codes:
            print("expected signer_quorum_shortfall in signer quorum negative policy output", file=sys.stderr)
            return 1
        if "custody_evidence_missing" not in quorum_no_go_reason_codes:
            print("expected custody_evidence_missing in signer quorum negative policy output", file=sys.stderr)
            return 1
        if quorum_no_go_policy.get("final_decision") != "NO-GO":
            print("expected signer quorum negative policy final_decision NO-GO", file=sys.stderr)
            return 1

    doc_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_custody_evidence_contract",
        "signer_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "Regression: #2226",
    ]
    ci_doc_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_custody_evidence_contract",
        "signer_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "Regression: #2226",
    ]
    readme_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_custody_evidence_contract",
        "signer_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "Regression: #2226",
    ]

    missing_markers: list[str] = []
    missing_markers.extend(
        ensure_markers_present(DOC_FILE.read_text(encoding="utf-8"), doc_markers, "docs/planning/kolme-devnet-ops.md")
    )
    missing_markers.extend(
        ensure_markers_present(CI_DOC_FILE.read_text(encoding="utf-8"), ci_doc_markers, "docs/ci/strategy.md")
    )
    missing_markers.extend(
        ensure_markers_present(README_FILE.read_text(encoding="utf-8"), readme_markers, "README.md")
    )
    if missing_markers:
        print(",".join(missing_markers), file=sys.stderr)
        return 1

    print("local Kolme live deployment preflight contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
