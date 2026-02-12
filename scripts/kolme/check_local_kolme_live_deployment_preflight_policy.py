#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

REQUIRED_RUNTIME_MODE = "kolme-live"
SIGNER_PROFILE_SELECTOR_ENV = "KAMN_KOLME_LIVE_SIGNER_PROFILE"
PRIMARY_SIGNER_PROFILE = "ops-primary"
SECONDARY_SIGNER_PROFILE = "ops-secondary"
PRIMARY_SIGNER_SECRET_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
SECONDARY_SIGNER_SECRET_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
FALLBACK_SIGNER_SECRET_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
REQUIRED_SECRET_HEX_LENGTH = 64
SIGNER_KEY_SOURCE_CONTRACT_VERSION = "v1"
QUORUM_EVIDENCE_SCHEMA_VERSION = "kamn.kolme.signer-quorum-evidence.v1"
ALLOWED_SIGNER_KEY_SOURCES = ("env-local", "managed-external")
ALLOWED_SIGNER_KEY_SOURCES_BY_PROFILE = {
    PRIMARY_SIGNER_PROFILE: ("env-local", "managed-external"),
    SECONDARY_SIGNER_PROFILE: ("env-local",),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local Kolme live deployment preflight summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
        reason_codes.append("schema_version_mismatch")

    mode = report.get("mode")
    if mode not in ("dry-run", "run"):
        reason_codes.append("mode_invalid")

    status = report.get("status")
    if status not in ("ok", "fail"):
        reason_codes.append("status_invalid")

    reason_code = report.get("reason_code")
    if not isinstance(reason_code, str) or not reason_code.strip():
        reason_codes.append("reason_code_missing")

    local_only_enforced = report.get("local_only_enforced")
    if not isinstance(local_only_enforced, bool):
        reason_codes.append("local_only_enforced_invalid")

    ci_fast_gate_eligible = report.get("ci_fast_gate_eligible")
    if not isinstance(ci_fast_gate_eligible, bool):
        reason_codes.append("ci_fast_gate_eligible_invalid")
    elif not ci_fast_gate_eligible:
        reason_codes.append("ci_fast_gate_eligibility_violation")

    elapsed_seconds = report.get("elapsed_seconds")
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        reason_codes.append("elapsed_seconds_invalid")

    max_seconds = report.get("max_seconds")
    if not isinstance(max_seconds, int) or max_seconds <= 0:
        reason_codes.append("max_seconds_invalid")

    budget_status = report.get("budget_status")
    if budget_status not in ("not_run", "within_budget", "exceeded_budget"):
        reason_codes.append("budget_status_invalid")

    runtime_mode = report.get("runtime_mode")
    if not isinstance(runtime_mode, str) or not runtime_mode.strip():
        reason_codes.append("runtime_mode_missing")
    elif runtime_mode != REQUIRED_RUNTIME_MODE:
        reason_codes.append("runtime_mode_mismatch")

    signer_profile_selector_env = report.get("signer_profile_selector_env")
    if not isinstance(signer_profile_selector_env, str) or not signer_profile_selector_env.strip():
        reason_codes.append("signer_profile_selector_env_missing")
    elif signer_profile_selector_env != SIGNER_PROFILE_SELECTOR_ENV:
        reason_codes.append("signer_profile_selector_env_mismatch")

    signer_profile = report.get("signer_profile")
    if not isinstance(signer_profile, str) or not signer_profile.strip():
        reason_codes.append("signer_profile_missing")
    elif signer_profile not in (PRIMARY_SIGNER_PROFILE, SECONDARY_SIGNER_PROFILE):
        reason_codes.append("signer_profile_mismatch")

    signer_private_key_env = report.get("signer_private_key_env")
    expected_signer_env = ""
    if signer_profile == PRIMARY_SIGNER_PROFILE:
        expected_signer_env = PRIMARY_SIGNER_SECRET_ENV
    elif signer_profile == SECONDARY_SIGNER_PROFILE:
        expected_signer_env = SECONDARY_SIGNER_SECRET_ENV
    if not isinstance(signer_private_key_env, str) or not signer_private_key_env.strip():
        reason_codes.append("signer_private_key_env_missing")
    elif expected_signer_env and signer_private_key_env != expected_signer_env:
        reason_codes.append("signer_private_key_env_mismatch")

    signer_key_source_contract_version = report.get("signer_key_source_contract_version")
    if not isinstance(signer_key_source_contract_version, str) or not signer_key_source_contract_version.strip():
        reason_codes.append("signer_key_source_contract_version_missing")
    elif signer_key_source_contract_version != SIGNER_KEY_SOURCE_CONTRACT_VERSION:
        reason_codes.append("signer_key_source_contract_version_mismatch")

    signer_key_source = report.get("signer_key_source")
    if not isinstance(signer_key_source, str) or not signer_key_source.strip():
        reason_codes.append("signer_key_source_missing")
    elif signer_key_source not in ALLOWED_SIGNER_KEY_SOURCES:
        reason_codes.append("signer_key_source_invalid")
    elif (
        isinstance(signer_profile, str)
        and signer_profile in ALLOWED_SIGNER_KEY_SOURCES_BY_PROFILE
        and signer_key_source not in ALLOWED_SIGNER_KEY_SOURCES_BY_PROFILE[signer_profile]
    ):
        reason_codes.append("signer_key_source_profile_pair_disallowed")

    signer_provenance_file = report.get("signer_provenance_file")
    if not isinstance(signer_provenance_file, str):
        reason_codes.append("signer_provenance_file_invalid")

    signer_provenance_present = report.get("signer_provenance_present")
    if not isinstance(signer_provenance_present, bool):
        reason_codes.append("signer_provenance_present_invalid")

    signer_provenance_sha256 = report.get("signer_provenance_sha256")
    if not isinstance(signer_provenance_sha256, str):
        reason_codes.append("signer_provenance_sha256_invalid")

    signer_provenance_sha256_valid = report.get("signer_provenance_sha256_valid")
    if not isinstance(signer_provenance_sha256_valid, bool):
        reason_codes.append("signer_provenance_sha256_valid_invalid")

    signer_rotation_epoch = report.get("signer_rotation_epoch")
    if not isinstance(signer_rotation_epoch, int) or signer_rotation_epoch <= 0:
        reason_codes.append("signer_rotation_epoch_invalid")

    signer_previous_rotation_epoch = report.get("signer_previous_rotation_epoch")
    if not isinstance(signer_previous_rotation_epoch, int) or signer_previous_rotation_epoch <= 0:
        reason_codes.append("signer_previous_rotation_epoch_invalid")

    signer_rotation_freshness_max_delta = report.get("signer_rotation_freshness_max_delta")
    if not isinstance(signer_rotation_freshness_max_delta, int) or signer_rotation_freshness_max_delta < 0:
        reason_codes.append("signer_rotation_freshness_max_delta_invalid")

    signer_rotation_delta_epochs = report.get("signer_rotation_delta_epochs")
    if not isinstance(signer_rotation_delta_epochs, int):
        reason_codes.append("signer_rotation_delta_epochs_invalid")
    elif signer_rotation_delta_epochs < 0:
        reason_codes.append("signer_rotation_epoch_invalid")
    elif (
        isinstance(signer_rotation_epoch, int)
        and signer_rotation_epoch > 0
        and isinstance(signer_previous_rotation_epoch, int)
        and signer_previous_rotation_epoch > 0
        and signer_rotation_delta_epochs != signer_rotation_epoch - signer_previous_rotation_epoch
    ):
        reason_codes.append("signer_rotation_delta_epochs_mismatch")
    elif (
        isinstance(signer_rotation_freshness_max_delta, int)
        and signer_rotation_freshness_max_delta >= 0
        and signer_rotation_delta_epochs > signer_rotation_freshness_max_delta
    ):
        reason_codes.append("signer_rotation_epoch_stale")

    signer_rotation_fresh = report.get("signer_rotation_fresh")
    if not isinstance(signer_rotation_fresh, bool):
        reason_codes.append("signer_rotation_fresh_invalid")
    elif "signer_rotation_epoch_stale" in reason_codes and signer_rotation_fresh:
        reason_codes.append("signer_rotation_fresh_contract_mismatch")

    fallback_signer_private_key_env = report.get("fallback_signer_private_key_env")
    if not isinstance(fallback_signer_private_key_env, str) or not fallback_signer_private_key_env.strip():
        reason_codes.append("fallback_signer_private_key_env_missing")
    elif fallback_signer_private_key_env != FALLBACK_SIGNER_SECRET_ENV:
        reason_codes.append("fallback_signer_private_key_env_mismatch")

    signer_secret_present = report.get("signer_secret_present")
    if not isinstance(signer_secret_present, bool):
        reason_codes.append("signer_secret_present_invalid")

    fallback_signer_secret_present = report.get("fallback_signer_secret_present")
    if not isinstance(fallback_signer_secret_present, bool):
        reason_codes.append("fallback_signer_secret_present_invalid")
    elif fallback_signer_secret_present:
        reason_codes.append("fallback_signer_secret_present_violation")

    signer_secret_hex_valid = report.get("signer_secret_hex_valid")
    if not isinstance(signer_secret_hex_valid, bool):
        reason_codes.append("signer_secret_hex_valid_invalid")

    required_approvals = report.get("required_approvals")
    if not isinstance(required_approvals, int) or required_approvals <= 0:
        reason_codes.append("required_approvals_invalid")

    received_approvals = report.get("received_approvals")
    if not isinstance(received_approvals, int) or received_approvals < 0:
        reason_codes.append("received_approvals_invalid")

    quorum_evidence_file = report.get("quorum_evidence_file")
    if not isinstance(quorum_evidence_file, str):
        reason_codes.append("quorum_evidence_file_invalid")

    quorum_evidence_present = report.get("quorum_evidence_present")
    if not isinstance(quorum_evidence_present, bool):
        reason_codes.append("quorum_evidence_present_invalid")

    quorum_evidence_sha256 = report.get("quorum_evidence_sha256")
    if not isinstance(quorum_evidence_sha256, str):
        reason_codes.append("quorum_evidence_sha256_invalid")

    quorum_evidence_sha256_valid = report.get("quorum_evidence_sha256_valid")
    if not isinstance(quorum_evidence_sha256_valid, bool):
        reason_codes.append("quorum_evidence_sha256_valid_invalid")

    quorum_evidence_schema_valid = report.get("quorum_evidence_schema_valid")
    if not isinstance(quorum_evidence_schema_valid, bool):
        reason_codes.append("quorum_evidence_schema_valid_invalid")

    quorum_evidence_approval_count = report.get("quorum_evidence_approval_count")
    if not isinstance(quorum_evidence_approval_count, int) or quorum_evidence_approval_count < 0:
        reason_codes.append("quorum_evidence_approval_count_invalid")

    quorum_evidence_signers_unique = report.get("quorum_evidence_signers_unique")
    if not isinstance(quorum_evidence_signers_unique, bool):
        reason_codes.append("quorum_evidence_signers_unique_invalid")

    quorum_evidence_matches_threshold = report.get("quorum_evidence_matches_threshold")
    if not isinstance(quorum_evidence_matches_threshold, bool):
        reason_codes.append("quorum_evidence_matches_threshold_invalid")

    quorum_evidence_custody_sha256_match = report.get("quorum_evidence_custody_sha256_match")
    if not isinstance(quorum_evidence_custody_sha256_match, bool):
        reason_codes.append("quorum_evidence_custody_sha256_match_invalid")

    custody_evidence_file = report.get("custody_evidence_file")
    if not isinstance(custody_evidence_file, str):
        reason_codes.append("custody_evidence_file_invalid")

    custody_evidence_present = report.get("custody_evidence_present")
    if not isinstance(custody_evidence_present, bool):
        reason_codes.append("custody_evidence_present_invalid")

    custody_evidence_sha256 = report.get("custody_evidence_sha256")
    if not isinstance(custody_evidence_sha256, str):
        reason_codes.append("custody_evidence_sha256_invalid")

    custody_evidence_sha256_valid = report.get("custody_evidence_sha256_valid")
    if not isinstance(custody_evidence_sha256_valid, bool):
        reason_codes.append("custody_evidence_sha256_valid_invalid")

    contracts = report.get("contracts")
    if not isinstance(contracts, dict):
        reason_codes.append("contracts_missing")
    else:
        if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
            reason_codes.append("ci_fast_gate_scope_mismatch")
        if contracts.get("required_runtime_mode") != REQUIRED_RUNTIME_MODE:
            reason_codes.append("required_runtime_mode_contract_mismatch")
        if contracts.get("signer_profile_selector_env") != SIGNER_PROFILE_SELECTOR_ENV:
            reason_codes.append("signer_profile_selector_env_contract_mismatch")
        if contracts.get("supported_signer_profiles") != [PRIMARY_SIGNER_PROFILE, SECONDARY_SIGNER_PROFILE]:
            reason_codes.append("supported_signer_profiles_contract_mismatch")
        if contracts.get("primary_signer_secret_env") != PRIMARY_SIGNER_SECRET_ENV:
            reason_codes.append("primary_signer_secret_env_contract_mismatch")
        if contracts.get("secondary_signer_secret_env") != SECONDARY_SIGNER_SECRET_ENV:
            reason_codes.append("secondary_signer_secret_env_contract_mismatch")
        if contracts.get("fallback_signer_secret_env") != FALLBACK_SIGNER_SECRET_ENV:
            reason_codes.append("fallback_signer_secret_env_contract_mismatch")
        if contracts.get("fallback_private_key_path_allowed") is not False:
            reason_codes.append("fallback_private_key_path_allowed_contract_mismatch")
        if contracts.get("required_secret_hex_length") != REQUIRED_SECRET_HEX_LENGTH:
            reason_codes.append("required_secret_hex_length_contract_mismatch")
        if contracts.get("secret_source") != "env":
            reason_codes.append("secret_source_contract_mismatch")
        approval_quorum_required = contracts.get("approval_quorum_required")
        if not isinstance(approval_quorum_required, int) or approval_quorum_required <= 0:
            reason_codes.append("approval_quorum_required_contract_invalid")
        elif isinstance(required_approvals, int) and approval_quorum_required != required_approvals:
            reason_codes.append("approval_quorum_required_contract_mismatch")
        if contracts.get("approval_quorum_source") != "local-operator-attestations":
            reason_codes.append("approval_quorum_source_contract_mismatch")
        if contracts.get("quorum_evidence_required") is not True:
            reason_codes.append("quorum_evidence_required_contract_mismatch")
        if contracts.get("quorum_evidence_sha256_required") is not True:
            reason_codes.append("quorum_evidence_sha256_required_contract_mismatch")
        if contracts.get("quorum_evidence_schema_version") != QUORUM_EVIDENCE_SCHEMA_VERSION:
            reason_codes.append("quorum_evidence_schema_version_contract_mismatch")
        if contracts.get("quorum_evidence_signer_uniqueness_required") is not True:
            reason_codes.append("quorum_evidence_signer_uniqueness_required_contract_mismatch")
        if contracts.get("quorum_evidence_custody_sha256_match_required") is not True:
            reason_codes.append("quorum_evidence_custody_sha256_match_required_contract_mismatch")
        if contracts.get("quorum_evidence_source") != "operator-attestation-bundle":
            reason_codes.append("quorum_evidence_source_contract_mismatch")
        if contracts.get("custody_evidence_required") is not True:
            reason_codes.append("custody_evidence_required_contract_mismatch")
        if contracts.get("custody_evidence_sha256_required") is not True:
            reason_codes.append("custody_evidence_sha256_required_contract_mismatch")
        if contracts.get("signer_provenance_required") is not True:
            reason_codes.append("signer_provenance_required_contract_mismatch")
        if contracts.get("signer_provenance_sha256_required") is not True:
            reason_codes.append("signer_provenance_sha256_required_contract_mismatch")
        if contracts.get("signer_key_source_contract_version") != SIGNER_KEY_SOURCE_CONTRACT_VERSION:
            reason_codes.append("signer_key_source_contract_version_contract_mismatch")
        if isinstance(signer_key_source, str) and signer_key_source and contracts.get("signer_key_source") != signer_key_source:
            reason_codes.append("signer_key_source_contract_mismatch")
        if contracts.get("signer_key_source_allowed_for_ops_primary") != ["env-local", "managed-external"]:
            reason_codes.append("signer_key_source_allowed_for_ops_primary_contract_mismatch")
        if contracts.get("signer_key_source_allowed_for_ops_secondary") != ["env-local"]:
            reason_codes.append("signer_key_source_allowed_for_ops_secondary_contract_mismatch")
        if (
            isinstance(signer_rotation_freshness_max_delta, int)
            and signer_rotation_freshness_max_delta >= 0
            and contracts.get("signer_rotation_freshness_max_delta") != signer_rotation_freshness_max_delta
        ):
            reason_codes.append("signer_rotation_freshness_max_delta_contract_mismatch")
        if contracts.get("signer_rotation_stale_rejected") is not True:
            reason_codes.append("signer_rotation_stale_rejected_contract_mismatch")

    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "runtime_mode_contract",
            "signer_profile_contract",
            "signer_secret_contract",
            "fallback_private_key_contract",
            "signer_quorum_contract",
            "quorum_evidence_contract",
            "custody_evidence_contract",
            "signer_provenance_contract",
            "signer_rotation_freshness_contract",
        }
        observed_ids: set[str] = set()
        for entry in checks:
            if not isinstance(entry, dict):
                reason_codes.append("check_entry_invalid")
                continue
            check_id = entry.get("id")
            command = entry.get("command")
            check_status = entry.get("status")
            check_reason_code = entry.get("reason_code")
            if not isinstance(check_id, str) or not check_id.strip():
                reason_codes.append("check_id_invalid")
                continue
            observed_ids.add(check_id)
            if not isinstance(command, str) or not command.strip():
                reason_codes.append(f"check_command_invalid:{check_id}")
            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            if not isinstance(check_reason_code, str) or not check_reason_code.strip():
                reason_codes.append(f"check_reason_code_invalid:{check_id}")
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "deployment_preflight_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
        if mode == "run":
            if signer_secret_present is not True:
                reason_codes.append("ok_status_signer_secret_presence_violation")
            if signer_secret_hex_valid is not True:
                reason_codes.append("ok_status_signer_secret_hex_violation")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "deployment_preflight_passed"):
            reason_codes.append("fail_status_reason_code_mismatch")

    if mode == "run":
        if isinstance(required_approvals, int) and isinstance(received_approvals, int):
            if received_approvals < required_approvals:
                reason_codes.append("signer_quorum_shortfall")
        if quorum_evidence_present is not True:
            reason_codes.append("quorum_evidence_missing")
        if quorum_evidence_sha256_valid is not True:
            reason_codes.append("quorum_evidence_sha256_invalid")
        if quorum_evidence_schema_valid is not True:
            reason_codes.append("quorum_evidence_schema_invalid")
        if quorum_evidence_signers_unique is not True:
            reason_codes.append("quorum_evidence_signers_not_unique")
        if quorum_evidence_matches_threshold is not True:
            reason_codes.append("quorum_evidence_approvals_mismatch")
        if quorum_evidence_custody_sha256_match is not True:
            reason_codes.append("quorum_evidence_custody_sha256_mismatch")
        if isinstance(quorum_evidence_file, str) and quorum_evidence_present is True and not quorum_evidence_file.strip():
            reason_codes.append("quorum_evidence_file_missing")
        if (
            isinstance(quorum_evidence_approval_count, int)
            and isinstance(received_approvals, int)
            and quorum_evidence_approval_count != received_approvals
        ):
            reason_codes.append("quorum_evidence_approval_count_mismatch")
        if custody_evidence_present is not True:
            reason_codes.append("custody_evidence_missing")
        if custody_evidence_sha256_valid is not True:
            reason_codes.append("custody_evidence_sha256_invalid")
        if isinstance(custody_evidence_file, str) and custody_evidence_present is True and not custody_evidence_file.strip():
            reason_codes.append("custody_evidence_file_missing")
        if signer_provenance_present is not True:
            reason_codes.append("signer_provenance_missing")
        if signer_provenance_sha256_valid is not True:
            reason_codes.append("signer_provenance_sha256_invalid")
        if isinstance(signer_provenance_file, str) and signer_provenance_present is True and not signer_provenance_file.strip():
            reason_codes.append("signer_provenance_file_missing")
        if (
            isinstance(signer_rotation_delta_epochs, int)
            and isinstance(signer_rotation_freshness_max_delta, int)
            and signer_rotation_delta_epochs > signer_rotation_freshness_max_delta
        ):
            reason_codes.append("signer_rotation_epoch_stale")
        if signer_rotation_fresh is not True:
            reason_codes.append("signer_rotation_fresh_violation")

    for required_reason_code in args.require_reason_code:
        if reason_code != required_reason_code:
            reason_codes.append(f"required_reason_code_missing:{required_reason_code}")

    if observed_final_decision and observed_final_decision != args.expected_final_decision:
        reason_codes.append("observed_final_decision_mismatch")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def main() -> int:
    args = parse_args()
    report_path = Path(args.report_file).resolve()
    report = json.loads(report_path.read_text(encoding="utf-8"))

    observed_status = report.get("status")
    observed_final_decision = ""
    if observed_status == "ok":
        observed_final_decision = "GO"
    elif observed_status == "fail":
        observed_final_decision = "NO-GO"

    final_decision, reason_codes = evaluate(report, args)
    output = {
        "schema_version": "kamn.kolme.local-live-deployment-preflight-policy-report.v1",
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_reason_codes": args.require_reason_code,
        "observed_status": observed_status,
        "observed_final_decision": observed_final_decision,
        "observed_reason_code": report.get("reason_code"),
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
