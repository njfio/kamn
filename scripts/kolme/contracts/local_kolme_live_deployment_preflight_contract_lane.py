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
    if summary.get("signer_profile_class") != "production":
        print("expected deployment preflight summary signer profile class marker", file=sys.stderr)
        return 1
    if summary.get("fallback_signer_secret_remediation") != "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
        print("expected deployment preflight summary fallback signer remediation marker", file=sys.stderr)
        return 1
    if summary.get("fallback_signer_secret_present") is not False:
        print("expected deployment preflight summary fallback signer secret marker to remain false", file=sys.stderr)
        return 1
    if summary.get("signer_key_source_contract_version") != "v1":
        print("expected deployment preflight summary signer key-source contract version marker", file=sys.stderr)
        return 1
    if summary.get("signer_key_source") != "env-local":
        print("expected deployment preflight summary signer key-source marker", file=sys.stderr)
        return 1
    if summary.get("signer_rotation_epoch") != 1:
        print("expected deployment preflight summary signer rotation epoch marker", file=sys.stderr)
        return 1
    if summary.get("signer_previous_rotation_epoch") != 1:
        print("expected deployment preflight summary signer previous rotation epoch marker", file=sys.stderr)
        return 1
    if summary.get("signer_rotation_freshness_max_delta") != 2:
        print("expected deployment preflight summary signer rotation freshness threshold marker", file=sys.stderr)
        return 1
    if summary.get("signer_rotation_delta_epochs") != 0:
        print("expected deployment preflight summary signer rotation delta marker", file=sys.stderr)
        return 1
    if summary.get("signer_rotation_fresh") is not False:
        print("expected deployment preflight summary signer rotation freshness marker false in dry-run", file=sys.stderr)
        return 1
    if summary.get("quorum_evidence_present") is not False:
        print("expected deployment preflight summary quorum evidence marker false in dry-run", file=sys.stderr)
        return 1
    if summary.get("quorum_evidence_matches_threshold") is not False:
        print("expected deployment preflight summary quorum threshold marker false in dry-run", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected deployment preflight summary runtime signer attestation schema marker", file=sys.stderr)
        return 1
    runtime_signer_attestation_bundle = summary.get("runtime_signer_attestation_bundle")
    if not isinstance(runtime_signer_attestation_bundle, dict):
        print("expected deployment preflight summary runtime signer attestation bundle", file=sys.stderr)
        return 1
    if runtime_signer_attestation_bundle.get("schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected deployment preflight summary runtime signer attestation bundle schema marker", file=sys.stderr)
        return 1
    if runtime_signer_attestation_bundle.get("required_approvals") != 2:
        print("expected deployment preflight summary runtime signer attestation required approvals marker", file=sys.stderr)
        return 1
    if runtime_signer_attestation_bundle.get("approved_signers") != ["ops-primary", "ops-secondary"]:
        print("expected deployment preflight summary runtime signer attestation approved signers marker", file=sys.stderr)
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
    if contracts.get("fallback_signer_secret_rejected_profile_class") != "production":
        print("expected deployment preflight contracts fallback signer rejection profile class marker", file=sys.stderr)
        return 1
    if contracts.get("fallback_signer_secret_rejected_profiles") != ["ops-primary", "ops-secondary"]:
        print("expected deployment preflight contracts fallback signer rejected profiles marker", file=sys.stderr)
        return 1
    if contracts.get("fallback_signer_secret_remediation") != "unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK":
        print("expected deployment preflight contracts fallback signer remediation marker", file=sys.stderr)
        return 1
    if contracts.get("fallback_signer_secret_rejection_reason_code") != "fallback_signer_secret_present_violation":
        print("expected deployment preflight contracts fallback signer rejection reason marker", file=sys.stderr)
        return 1
    if contracts.get("fallback_signer_secret_checkpoint_reason_code") != "checkpoint_failed_fallback_private_key_contract":
        print("expected deployment preflight contracts fallback signer checkpoint reason marker", file=sys.stderr)
        return 1
    if contracts.get("approval_quorum_required") != 2:
        print("expected deployment preflight contracts approval_quorum_required=2", file=sys.stderr)
        return 1
    if contracts.get("quorum_evidence_required") is not True:
        print("expected deployment preflight contracts quorum_evidence_required=true", file=sys.stderr)
        return 1
    if contracts.get("quorum_evidence_sha256_required") is not True:
        print("expected deployment preflight contracts quorum_evidence_sha256_required=true", file=sys.stderr)
        return 1
    if contracts.get("quorum_evidence_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected deployment preflight contracts quorum_evidence_schema_version marker", file=sys.stderr)
        return 1
    if contracts.get("quorum_evidence_signer_uniqueness_required") is not True:
        print("expected deployment preflight contracts quorum_evidence_signer_uniqueness_required=true", file=sys.stderr)
        return 1
    if contracts.get("quorum_evidence_custody_sha256_match_required") is not True:
        print("expected deployment preflight contracts quorum_evidence_custody_sha256_match_required=true", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected deployment preflight contracts runtime signer attestation schema marker", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_signer_uniqueness_required") is not True:
        print("expected deployment preflight contracts runtime signer attestation uniqueness marker", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_threshold_required") is not True:
        print("expected deployment preflight contracts runtime signer attestation threshold marker", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_profile_membership_required") is not True:
        print("expected deployment preflight contracts runtime signer attestation profile membership marker", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_required_approvals") != 2:
        print("expected deployment preflight contracts runtime signer attestation required approvals marker", file=sys.stderr)
        return 1
    if contracts.get("custody_evidence_required") is not True:
        print("expected deployment preflight contracts custody_evidence_required=true", file=sys.stderr)
        return 1
    if contracts.get("signer_provenance_required") is not True:
        print("expected deployment preflight contracts signer_provenance_required=true", file=sys.stderr)
        return 1
    if contracts.get("signer_provenance_sha256_required") is not True:
        print("expected deployment preflight contracts signer_provenance_sha256_required=true", file=sys.stderr)
        return 1
    if contracts.get("signer_key_source_contract_version") != "v1":
        print("expected deployment preflight contracts signer_key_source_contract_version=v1", file=sys.stderr)
        return 1
    if contracts.get("signer_key_source") != "env-local":
        print("expected deployment preflight contracts signer_key_source=env-local", file=sys.stderr)
        return 1
    if contracts.get("signer_rotation_freshness_max_delta") != 2:
        print("expected deployment preflight contracts signer_rotation_freshness_max_delta=2", file=sys.stderr)
        return 1
    if contracts.get("signer_rotation_stale_rejected") is not True:
        print("expected deployment preflight contracts signer_rotation_stale_rejected=true", file=sys.stderr)
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
        if "fallback_signer_secret_checkpoint_reason_mismatch" not in no_go_reason_codes:
            print(
                "expected fallback signer secret checkpoint reason mismatch in deployment preflight negative policy output",
                file=sys.stderr,
            )
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

        quorum_evidence_negative_report = temp_path / "quorum_evidence_negative_summary.json"
        quorum_evidence_negative_policy = temp_path / "quorum_evidence_negative_policy.json"
        quorum_evidence_negative_summary = dict(summary)
        quorum_evidence_negative_summary["mode"] = "run"
        quorum_evidence_negative_summary["status"] = "fail"
        quorum_evidence_negative_summary["reason_code"] = "checkpoint_failed_quorum_evidence_contract"
        quorum_evidence_negative_summary["signer_secret_present"] = True
        quorum_evidence_negative_summary["signer_secret_hex_valid"] = True
        quorum_evidence_negative_summary["required_approvals"] = 2
        quorum_evidence_negative_summary["received_approvals"] = 2
        quorum_evidence_negative_summary["custody_evidence_file"] = "/tmp/custody-evidence.json"
        quorum_evidence_negative_summary["custody_evidence_present"] = True
        quorum_evidence_negative_summary["custody_evidence_sha256"] = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        quorum_evidence_negative_summary["custody_evidence_sha256_valid"] = True
        quorum_evidence_negative_summary["quorum_evidence_file"] = ""
        quorum_evidence_negative_summary["quorum_evidence_present"] = False
        quorum_evidence_negative_summary["quorum_evidence_sha256"] = ""
        quorum_evidence_negative_summary["quorum_evidence_sha256_valid"] = False
        quorum_evidence_negative_summary["quorum_evidence_schema_valid"] = False
        quorum_evidence_negative_summary["quorum_evidence_approval_count"] = 0
        quorum_evidence_negative_summary["quorum_evidence_signers_unique"] = False
        quorum_evidence_negative_summary["quorum_evidence_matches_threshold"] = False
        quorum_evidence_negative_summary["quorum_evidence_custody_sha256_match"] = False
        quorum_evidence_negative_summary["runtime_signer_attestation_bundle"] = {
            "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
            "required_approvals": 2,
            "approved_signers": ["ops-primary"],
            "signer_profile": "ops-primary",
            "signer_key_source": "env-local",
        }
        quorum_evidence_negative_summary["runtime_signer_attestation_profile_approved"] = True
        quorum_evidence_negative_report.write_text(
            json.dumps(quorum_evidence_negative_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        quorum_evidence_no_go_result = run_policy_check(
            report_file=quorum_evidence_negative_report,
            output_json=quorum_evidence_negative_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_quorum_evidence_contract",
        )
        if quorum_evidence_no_go_result.returncode == 0:
            print("expected quorum evidence negative proof to fail closed", file=sys.stderr)
            return 1
        quorum_evidence_no_go_policy = json.loads(quorum_evidence_negative_policy.read_text(encoding="utf-8"))
        quorum_evidence_no_go_reason_codes = quorum_evidence_no_go_policy.get("reason_codes")
        if not isinstance(quorum_evidence_no_go_reason_codes, list):
            print("expected reason_codes list in quorum evidence negative policy output", file=sys.stderr)
            return 1
        if "quorum_evidence_missing" not in quorum_evidence_no_go_reason_codes:
            print("expected quorum_evidence_missing in quorum evidence negative policy output", file=sys.stderr)
            return 1
        if "quorum_evidence_approvals_mismatch" not in quorum_evidence_no_go_reason_codes:
            print("expected quorum_evidence_approvals_mismatch in quorum evidence negative policy output", file=sys.stderr)
            return 1
        if "runtime_signer_attestation_quorum_shortfall" not in quorum_evidence_no_go_reason_codes:
            print(
                "expected runtime_signer_attestation_quorum_shortfall in quorum evidence negative policy output",
                file=sys.stderr,
            )
            return 1
        if quorum_evidence_no_go_policy.get("final_decision") != "NO-GO":
            print("expected quorum evidence negative policy final_decision NO-GO", file=sys.stderr)
            return 1

        attestation_duplicate_report = temp_path / "attestation_duplicate_summary.json"
        attestation_duplicate_policy = temp_path / "attestation_duplicate_policy.json"
        attestation_duplicate_summary = dict(summary)
        attestation_duplicate_summary["mode"] = "run"
        attestation_duplicate_summary["status"] = "fail"
        attestation_duplicate_summary["reason_code"] = "checkpoint_failed_quorum_evidence_contract"
        attestation_duplicate_summary["signer_secret_present"] = True
        attestation_duplicate_summary["signer_secret_hex_valid"] = True
        attestation_duplicate_summary["required_approvals"] = 2
        attestation_duplicate_summary["received_approvals"] = 2
        attestation_duplicate_summary["custody_evidence_file"] = "/tmp/custody-evidence.json"
        attestation_duplicate_summary["custody_evidence_present"] = True
        attestation_duplicate_summary["custody_evidence_sha256"] = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        attestation_duplicate_summary["custody_evidence_sha256_valid"] = True
        attestation_duplicate_summary["quorum_evidence_file"] = "/tmp/attestation-duplicate.json"
        attestation_duplicate_summary["quorum_evidence_present"] = True
        attestation_duplicate_summary["quorum_evidence_sha256"] = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        attestation_duplicate_summary["quorum_evidence_sha256_valid"] = True
        attestation_duplicate_summary["quorum_evidence_schema_valid"] = True
        attestation_duplicate_summary["quorum_evidence_approval_count"] = 2
        attestation_duplicate_summary["quorum_evidence_signers_unique"] = False
        attestation_duplicate_summary["quorum_evidence_matches_threshold"] = True
        attestation_duplicate_summary["quorum_evidence_custody_sha256_match"] = True
        attestation_duplicate_summary["runtime_signer_attestation_bundle"] = {
            "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
            "required_approvals": 2,
            "approved_signers": ["ops-primary", "ops-primary"],
            "signer_profile": "ops-primary",
            "signer_key_source": "env-local",
        }
        attestation_duplicate_summary["runtime_signer_attestation_profile_approved"] = True
        attestation_duplicate_report.write_text(
            json.dumps(attestation_duplicate_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        attestation_duplicate_result = run_policy_check(
            report_file=attestation_duplicate_report,
            output_json=attestation_duplicate_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_quorum_evidence_contract",
        )
        if attestation_duplicate_result.returncode == 0:
            print("expected duplicate signer attestation proof to fail closed", file=sys.stderr)
            return 1
        attestation_duplicate_policy_payload = json.loads(attestation_duplicate_policy.read_text(encoding="utf-8"))
        attestation_duplicate_reason_codes = attestation_duplicate_policy_payload.get("reason_codes")
        if not isinstance(attestation_duplicate_reason_codes, list):
            print("expected reason_codes list in duplicate signer attestation policy output", file=sys.stderr)
            return 1
        if "runtime_signer_attestation_approved_signers_not_unique" not in attestation_duplicate_reason_codes:
            print(
                "expected runtime_signer_attestation_approved_signers_not_unique in duplicate signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if attestation_duplicate_policy_payload.get("final_decision") != "NO-GO":
            print("expected duplicate signer attestation policy final_decision NO-GO", file=sys.stderr)
            return 1

        attestation_schema_invalid_report = temp_path / "attestation_schema_invalid_summary.json"
        attestation_schema_invalid_policy = temp_path / "attestation_schema_invalid_policy.json"
        attestation_schema_invalid_summary = dict(summary)
        attestation_schema_invalid_summary["mode"] = "run"
        attestation_schema_invalid_summary["status"] = "fail"
        attestation_schema_invalid_summary["reason_code"] = "checkpoint_failed_quorum_evidence_contract"
        attestation_schema_invalid_summary["signer_secret_present"] = True
        attestation_schema_invalid_summary["signer_secret_hex_valid"] = True
        attestation_schema_invalid_summary["required_approvals"] = 2
        attestation_schema_invalid_summary["received_approvals"] = 2
        attestation_schema_invalid_summary["custody_evidence_file"] = "/tmp/custody-evidence.json"
        attestation_schema_invalid_summary["custody_evidence_present"] = True
        attestation_schema_invalid_summary["custody_evidence_sha256"] = (
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        )
        attestation_schema_invalid_summary["custody_evidence_sha256_valid"] = True
        attestation_schema_invalid_summary["quorum_evidence_file"] = "/tmp/attestation-schema-invalid.json"
        attestation_schema_invalid_summary["quorum_evidence_present"] = True
        attestation_schema_invalid_summary["quorum_evidence_sha256"] = (
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        )
        attestation_schema_invalid_summary["quorum_evidence_sha256_valid"] = True
        attestation_schema_invalid_summary["quorum_evidence_schema_valid"] = True
        attestation_schema_invalid_summary["quorum_evidence_approval_count"] = 2
        attestation_schema_invalid_summary["quorum_evidence_signers_unique"] = True
        attestation_schema_invalid_summary["quorum_evidence_matches_threshold"] = True
        attestation_schema_invalid_summary["quorum_evidence_custody_sha256_match"] = True
        attestation_schema_invalid_summary["runtime_signer_attestation_bundle"] = {
            "schema_version": "kamn.kolme.runtime-signer-attestation.v0",
            "required_approvals": 2,
            "approved_signers": ["ops-primary", "ops-secondary"],
            "signer_profile": "ops-primary",
            "signer_key_source": "env-local",
        }
        attestation_schema_invalid_summary["runtime_signer_attestation_profile_approved"] = True
        attestation_schema_invalid_report.write_text(
            json.dumps(attestation_schema_invalid_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        attestation_schema_invalid_result = run_policy_check(
            report_file=attestation_schema_invalid_report,
            output_json=attestation_schema_invalid_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_quorum_evidence_contract",
        )
        if attestation_schema_invalid_result.returncode == 0:
            print("expected schema-invalid signer attestation proof to fail closed", file=sys.stderr)
            return 1
        attestation_schema_invalid_policy_payload = json.loads(
            attestation_schema_invalid_policy.read_text(encoding="utf-8")
        )
        attestation_schema_invalid_reason_codes = attestation_schema_invalid_policy_payload.get("reason_codes")
        if not isinstance(attestation_schema_invalid_reason_codes, list):
            print("expected reason_codes list in schema-invalid signer attestation policy output", file=sys.stderr)
            return 1
        if "runtime_signer_attestation_schema_invalid" not in attestation_schema_invalid_reason_codes:
            print(
                "expected runtime_signer_attestation_schema_invalid in schema-invalid signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if attestation_schema_invalid_policy_payload.get("final_decision") != "NO-GO":
            print("expected schema-invalid signer attestation policy final_decision NO-GO", file=sys.stderr)
            return 1

        provenance_negative_report = temp_path / "signer_provenance_negative_summary.json"
        provenance_negative_policy = temp_path / "signer_provenance_negative_policy.json"
        provenance_negative_summary = dict(summary)
        provenance_negative_summary["mode"] = "run"
        provenance_negative_summary["status"] = "fail"
        provenance_negative_summary["reason_code"] = "checkpoint_failed_signer_provenance_contract"
        provenance_negative_summary["signer_secret_present"] = True
        provenance_negative_summary["signer_secret_hex_valid"] = True
        provenance_negative_summary["required_approvals"] = 2
        provenance_negative_summary["received_approvals"] = 2
        provenance_negative_summary["custody_evidence_file"] = "/tmp/custody-evidence.json"
        provenance_negative_summary["custody_evidence_present"] = True
        provenance_negative_summary["custody_evidence_sha256"] = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        provenance_negative_summary["custody_evidence_sha256_valid"] = True
        provenance_negative_summary["signer_provenance_file"] = ""
        provenance_negative_summary["signer_provenance_present"] = False
        provenance_negative_summary["signer_provenance_sha256"] = ""
        provenance_negative_summary["signer_provenance_sha256_valid"] = False
        provenance_negative_summary["signer_key_source_contract_version"] = "v0"
        provenance_negative_summary["signer_key_source"] = "legacy-local"
        provenance_negative_summary["signer_rotation_epoch"] = 1
        provenance_negative_summary["signer_previous_rotation_epoch"] = 1
        provenance_negative_summary["signer_rotation_freshness_max_delta"] = 2
        provenance_negative_summary["signer_rotation_delta_epochs"] = 0
        provenance_negative_summary["signer_rotation_fresh"] = False
        provenance_negative_report.write_text(
            json.dumps(provenance_negative_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        provenance_no_go_result = run_policy_check(
            report_file=provenance_negative_report,
            output_json=provenance_negative_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_signer_provenance_contract",
        )
        if provenance_no_go_result.returncode == 0:
            print("expected signer provenance negative proof to fail closed", file=sys.stderr)
            return 1
        provenance_no_go_policy = json.loads(provenance_negative_policy.read_text(encoding="utf-8"))
        provenance_no_go_reason_codes = provenance_no_go_policy.get("reason_codes")
        if not isinstance(provenance_no_go_reason_codes, list):
            print("expected reason_codes list in signer provenance negative policy output", file=sys.stderr)
            return 1
        if "signer_provenance_missing" not in provenance_no_go_reason_codes:
            print("expected signer_provenance_missing in signer provenance negative policy output", file=sys.stderr)
            return 1
        if "signer_key_source_contract_version_mismatch" not in provenance_no_go_reason_codes:
            print("expected signer_key_source_contract_version_mismatch in signer provenance negative policy output", file=sys.stderr)
            return 1
        if provenance_no_go_policy.get("final_decision") != "NO-GO":
            print("expected signer provenance negative policy final_decision NO-GO", file=sys.stderr)
            return 1

        rotation_negative_report = temp_path / "signer_rotation_negative_summary.json"
        rotation_negative_policy = temp_path / "signer_rotation_negative_policy.json"
        rotation_negative_summary = dict(summary)
        rotation_negative_summary["mode"] = "run"
        rotation_negative_summary["status"] = "fail"
        rotation_negative_summary["reason_code"] = "checkpoint_failed_signer_rotation_freshness_contract"
        rotation_negative_summary["signer_secret_present"] = True
        rotation_negative_summary["signer_secret_hex_valid"] = True
        rotation_negative_summary["required_approvals"] = 2
        rotation_negative_summary["received_approvals"] = 2
        rotation_negative_summary["custody_evidence_file"] = "/tmp/custody-evidence.json"
        rotation_negative_summary["custody_evidence_present"] = True
        rotation_negative_summary["custody_evidence_sha256"] = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        rotation_negative_summary["custody_evidence_sha256_valid"] = True
        rotation_negative_summary["signer_provenance_file"] = "/tmp/provenance-evidence.json"
        rotation_negative_summary["signer_provenance_present"] = True
        rotation_negative_summary["signer_provenance_sha256"] = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        rotation_negative_summary["signer_provenance_sha256_valid"] = True
        rotation_negative_summary["signer_key_source_contract_version"] = "v1"
        rotation_negative_summary["signer_key_source"] = "env-local"
        rotation_negative_summary["signer_rotation_epoch"] = 8
        rotation_negative_summary["signer_previous_rotation_epoch"] = 3
        rotation_negative_summary["signer_rotation_freshness_max_delta"] = 2
        rotation_negative_summary["signer_rotation_delta_epochs"] = 5
        rotation_negative_summary["signer_rotation_fresh"] = False
        rotation_negative_report.write_text(
            json.dumps(rotation_negative_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        rotation_no_go_result = run_policy_check(
            report_file=rotation_negative_report,
            output_json=rotation_negative_policy,
            expected_final_decision="NO-GO",
            required_reason_code="checkpoint_failed_signer_rotation_freshness_contract",
        )
        if rotation_no_go_result.returncode == 0:
            print("expected signer rotation negative proof to fail closed", file=sys.stderr)
            return 1
        rotation_no_go_policy = json.loads(rotation_negative_policy.read_text(encoding="utf-8"))
        rotation_no_go_reason_codes = rotation_no_go_policy.get("reason_codes")
        if not isinstance(rotation_no_go_reason_codes, list):
            print("expected reason_codes list in signer rotation negative policy output", file=sys.stderr)
            return 1
        if "signer_rotation_epoch_stale" not in rotation_no_go_reason_codes:
            print("expected signer_rotation_epoch_stale in signer rotation negative policy output", file=sys.stderr)
            return 1
        if "signer_rotation_fresh_violation" not in rotation_no_go_reason_codes:
            print("expected signer_rotation_fresh_violation in signer rotation negative policy output", file=sys.stderr)
            return 1
        if rotation_no_go_policy.get("final_decision") != "NO-GO":
            print("expected signer rotation negative policy final_decision NO-GO", file=sys.stderr)
            return 1

    doc_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_mismatch",
        "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_rejected_profile_class=production",
        "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_quorum_evidence_contract",
        "checkpoint_failed_custody_evidence_contract",
        "checkpoint_failed_signer_provenance_contract",
        "checkpoint_failed_signer_rotation_freshness_contract",
        "signer_quorum_shortfall",
        "quorum_evidence_missing",
        "quorum_evidence_approvals_mismatch",
        "quorum_evidence_custody_sha256_mismatch",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "signer_key_source_contract_version",
        "signer_key_source",
        "signer_provenance_file",
        "signer_rotation_epoch_stale",
        "Regression: #2226",
        "Regression: #2300",
        "Regression: #2301",
        "Regression: #2326",
        "Regression: #2327",
    ]
    ci_doc_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_mismatch",
        "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_rejected_profile_class=production",
        "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_quorum_evidence_contract",
        "checkpoint_failed_custody_evidence_contract",
        "checkpoint_failed_signer_provenance_contract",
        "checkpoint_failed_signer_rotation_freshness_contract",
        "signer_quorum_shortfall",
        "quorum_evidence_missing",
        "quorum_evidence_approvals_mismatch",
        "quorum_evidence_custody_sha256_mismatch",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "signer_key_source_contract_version",
        "signer_key_source",
        "signer_provenance_file",
        "signer_rotation_epoch_stale",
        "Regression: #2226",
        "Regression: #2300",
        "Regression: #2301",
        "Regression: #2326",
        "Regression: #2327",
    ]
    readme_markers = [
        "run_local_kolme_live_deployment_preflight_lane.sh",
        "check_local_kolme_live_deployment_preflight_policy.py",
        "run_local_kolme_live_deployment_preflight_contract_lane.sh",
        "runtime_mode_mismatch",
        "checkpoint_failed_signer_secret_contract",
        "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_mismatch",
        "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_rejected_profile_class=production",
        "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract",
        "checkpoint_failed_signer_quorum_contract",
        "checkpoint_failed_quorum_evidence_contract",
        "checkpoint_failed_custody_evidence_contract",
        "checkpoint_failed_signer_provenance_contract",
        "checkpoint_failed_signer_rotation_freshness_contract",
        "signer_quorum_shortfall",
        "quorum_evidence_missing",
        "quorum_evidence_approvals_mismatch",
        "quorum_evidence_custody_sha256_mismatch",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "custody_evidence_missing",
        "custody_evidence_sha256_invalid",
        "signer_key_source_contract_version",
        "signer_key_source",
        "signer_provenance_file",
        "signer_rotation_epoch_stale",
        "Regression: #2226",
        "Regression: #2300",
        "Regression: #2301",
        "Regression: #2326",
        "Regression: #2327",
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
