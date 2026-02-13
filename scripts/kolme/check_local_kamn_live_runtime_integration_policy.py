#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
RUNTIME_SIGNER_PROFILE_PRIMARY = "ops-primary"
RUNTIME_SIGNER_PROFILE_SECONDARY = "ops-secondary"
ALLOWED_RUNTIME_SIGNER_PROFILES = (
    RUNTIME_SIGNER_PROFILE_PRIMARY,
    RUNTIME_SIGNER_PROFILE_SECONDARY,
)
ALLOWED_RUNTIME_SIGNER_KEY_SOURCES = ("env-local", "managed-external")
RUNTIME_SIGNER_PRIVATE_KEY_ENV_BY_PROFILE = {
    RUNTIME_SIGNER_PROFILE_PRIMARY: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    RUNTIME_SIGNER_PROFILE_SECONDARY: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
}
RUNTIME_SIGNER_KEY_REF_ENV_BY_PROFILE = {
    RUNTIME_SIGNER_PROFILE_PRIMARY: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    RUNTIME_SIGNER_PROFILE_SECONDARY: "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
}
RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION = "kamn.kolme.runtime-signer-attestation.v1"
RUNTIME_REAL_SIGNING_PROFILE_VALUE = "kolme-fork-secp256k1-v1"
RUNTIME_REAL_SIGNING_PROFILE_MARKER = (
    f"KAMN_KOLME_LIVE_SIGNING_PROFILE={RUNTIME_REAL_SIGNING_PROFILE_VALUE}"
)


ALLOWED_RUNTIME_COMMIT_FAILURE_TAXONOMIES = {
    "none",
    "policy.rejected",
    "budget.exceeded",
    "transport.preflight.timeout",
    "transport.preflight.failed",
    "transport.submit.timeout",
    "transport.submit.failed",
    "finality.timeout",
    "finality.failed",
    "runtime.summary.unavailable",
    "runtime.unknown",
}

NESTED_RUNTIME_REASON_TO_TAXONOMY = {
    "live_preflight_timeout": "transport.preflight.timeout",
    "live_preflight_failed": "transport.preflight.failed",
    "live_runtime_commit_command_timeout": "transport.submit.timeout",
    "live_runtime_commit_command_failed": "transport.submit.failed",
    "live_finality_command_timeout": "finality.timeout",
    "live_finality_command_failed": "finality.failed",
    "live_runtime_commit_budget_exceeded": "budget.exceeded",
}


def evaluate_runtime_signer_attestation_bundle(
    attestation_bundle: object, runtime_signer_profile: object
) -> list[str]:
    reason_codes: list[str] = []
    if not isinstance(attestation_bundle, dict):
        return ["runtime_signer_attestation_bundle_missing"]

    if attestation_bundle.get("schema_version") != RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION:
        reason_codes.append("runtime_signer_attestation_schema_invalid")

    required_approvals = attestation_bundle.get("required_approvals")
    if not isinstance(required_approvals, int) or required_approvals <= 0:
        reason_codes.append("runtime_signer_attestation_required_approvals_invalid")

    approved_signers = attestation_bundle.get("approved_signers")
    normalized_signers: list[str] = []
    if not isinstance(approved_signers, list) or not approved_signers:
        reason_codes.append("runtime_signer_attestation_approved_signers_invalid")
    else:
        for entry in approved_signers:
            if not isinstance(entry, str) or not entry.strip():
                reason_codes.append("runtime_signer_attestation_approved_signers_invalid")
                break
            normalized_signers.append(entry.strip())
        if len(set(normalized_signers)) != len(normalized_signers):
            reason_codes.append("runtime_signer_attestation_approved_signers_not_unique")
        if isinstance(required_approvals, int) and len(normalized_signers) < required_approvals:
            reason_codes.append("runtime_signer_attestation_quorum_shortfall")
        if (
            isinstance(runtime_signer_profile, str)
            and runtime_signer_profile.strip()
            and runtime_signer_profile not in normalized_signers
        ):
            reason_codes.append("runtime_signer_attestation_profile_not_approved")

    return reason_codes


def expected_runtime_commit_failure_taxonomy(
    status: object,
    reason_code: object,
    runtime_commit_reason_code: object,
    runtime_commit_policy_reason_code: object,
    runtime_commit_nested_reason_code: object,
) -> str:
    if status == "ok":
        return "none"

    if reason_code == "runtime_commit_policy_failed":
        return "policy.rejected"

    if reason_code == "runtime_integration_budget_exceeded":
        return "budget.exceeded"

    if reason_code != "runtime_commit_endpoint_failed":
        return "none"

    if runtime_commit_reason_code == "runtime_commit_endpoint_timeout":
        return "transport.submit.timeout"

    if (
        runtime_commit_policy_reason_code == "runtime_commit_endpoint_failed"
        and runtime_commit_nested_reason_code in ("report_missing", "report_invalid_json", "reason_code_missing")
    ):
        return "runtime.summary.unavailable"

    if isinstance(runtime_commit_nested_reason_code, str):
        return NESTED_RUNTIME_REASON_TO_TAXONOMY.get(runtime_commit_nested_reason_code, "runtime.unknown")
    return "runtime.unknown"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local KAMN live runtime integration summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
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

    if report.get("local_only_enforced") is not True:
        reason_codes.append("local_only_enforced_missing")

    ci_fast_gate_eligible = report.get("ci_fast_gate_eligible")
    if not isinstance(ci_fast_gate_eligible, bool):
        reason_codes.append("ci_fast_gate_eligible_invalid")
    elif ci_fast_gate_eligible is True:
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

    runtime_commit_command = report.get("runtime_commit_command")
    if not isinstance(runtime_commit_command, str) or not runtime_commit_command.strip():
        reason_codes.append("runtime_commit_command_missing")
    else:
        if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in runtime_commit_command:
            reason_codes.append("runtime_commit_contract_lane_missing")
        if "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider" not in runtime_commit_command:
            reason_codes.append("runtime_provider_contract_marker_missing")
        if RUNTIME_REAL_SIGNING_PROFILE_MARKER not in runtime_commit_command:
            reason_codes.append("runtime_commit_real_signing_profile_marker_missing")
        if "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated" in runtime_commit_command:
            reason_codes.append("runtime_commit_simulated_signing_profile_detected")

    runtime_provider_client_contract = report.get("runtime_provider_client_contract")
    if not isinstance(runtime_provider_client_contract, str) or not runtime_provider_client_contract.strip():
        reason_codes.append("runtime_provider_client_contract_missing")
    elif runtime_provider_client_contract != "KolmeRuntimeCommitLiveProvider":
        reason_codes.append("runtime_provider_client_contract_mismatch")

    runtime_signing_profile = report.get("runtime_signing_profile")
    if not isinstance(runtime_signing_profile, str) or not runtime_signing_profile.strip():
        reason_codes.append("runtime_signing_profile_missing")
    elif runtime_signing_profile != RUNTIME_REAL_SIGNING_PROFILE_VALUE:
        reason_codes.append("runtime_signing_profile_mismatch")

    runtime_profile = report.get("runtime_profile")
    if not isinstance(runtime_profile, str) or not runtime_profile.strip():
        reason_codes.append("runtime_profile_missing")
    elif runtime_profile not in ("standard", "real-node"):
        reason_codes.append("runtime_profile_invalid")
    elif mode == "run" and runtime_profile != "real-node":
        reason_codes.append("runtime_profile_run_mode_mismatch")

    runtime_signer_profile = report.get("runtime_signer_profile")
    expected_runtime_signer_private_key_env = ""
    expected_runtime_signer_key_reference_env = ""
    if not isinstance(runtime_signer_profile, str) or not runtime_signer_profile.strip():
        reason_codes.append("runtime_signer_profile_missing")
    elif runtime_signer_profile not in ALLOWED_RUNTIME_SIGNER_PROFILES:
        reason_codes.append("runtime_signer_profile_invalid")
    else:
        expected_runtime_signer_private_key_env = RUNTIME_SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[runtime_signer_profile]
        expected_runtime_signer_key_reference_env = RUNTIME_SIGNER_KEY_REF_ENV_BY_PROFILE[runtime_signer_profile]

    runtime_signer_key_source = report.get("runtime_signer_key_source")
    normalized_runtime_signer_key_source = ""
    if not isinstance(runtime_signer_key_source, str) or not runtime_signer_key_source.strip():
        reason_codes.append("runtime_signer_key_source_missing")
    else:
        normalized_runtime_signer_key_source = runtime_signer_key_source.strip()
        if normalized_runtime_signer_key_source not in ALLOWED_RUNTIME_SIGNER_KEY_SOURCES:
            reason_codes.append("runtime_signer_key_source_invalid")
        if (
            normalized_runtime_signer_key_source == "managed-external"
            and runtime_signer_profile == RUNTIME_SIGNER_PROFILE_SECONDARY
        ):
            reason_codes.append("runtime_signer_key_source_profile_pair_disallowed")

    runtime_signer_private_key_env = report.get("runtime_signer_private_key_env")
    if not isinstance(runtime_signer_private_key_env, str) or not runtime_signer_private_key_env.strip():
        reason_codes.append("runtime_signer_private_key_env_missing")
    elif (
        expected_runtime_signer_private_key_env
        and runtime_signer_private_key_env != expected_runtime_signer_private_key_env
    ):
        reason_codes.append("runtime_signer_private_key_env_mismatch")

    runtime_signer_key_reference_env = report.get("runtime_signer_key_reference_env")
    if not isinstance(runtime_signer_key_reference_env, str) or not runtime_signer_key_reference_env.strip():
        reason_codes.append("runtime_signer_key_reference_env_missing")
    elif (
        expected_runtime_signer_key_reference_env
        and runtime_signer_key_reference_env != expected_runtime_signer_key_reference_env
    ):
        reason_codes.append("runtime_signer_key_reference_env_mismatch")

    runtime_signer_raw_private_key_present = report.get("runtime_signer_raw_private_key_present")
    if not isinstance(runtime_signer_raw_private_key_present, bool):
        reason_codes.append("runtime_signer_raw_private_key_present_invalid")
    elif normalized_runtime_signer_key_source == "managed-external" and runtime_signer_raw_private_key_present:
        reason_codes.append("runtime_signer_managed_external_raw_private_key_present_violation")

    runtime_signer_attestation_schema_version = report.get("runtime_signer_attestation_schema_version")
    if (
        not isinstance(runtime_signer_attestation_schema_version, str)
        or not runtime_signer_attestation_schema_version.strip()
    ):
        reason_codes.append("runtime_signer_attestation_schema_version_missing")
    elif runtime_signer_attestation_schema_version != RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION:
        reason_codes.append("runtime_signer_attestation_schema_version_mismatch")

    reason_codes.extend(
        evaluate_runtime_signer_attestation_bundle(
            report.get("runtime_signer_attestation_bundle"),
            runtime_signer_profile,
        )
    )

    runtime_commit_live_policy_report = report.get("runtime_commit_live_policy_report")
    if not isinstance(runtime_commit_live_policy_report, str) or not runtime_commit_live_policy_report.strip():
        reason_codes.append("runtime_commit_live_policy_report_missing")

    runtime_commit_reason_code = report.get("runtime_commit_reason_code")
    if not isinstance(runtime_commit_reason_code, str) or not runtime_commit_reason_code.strip():
        reason_codes.append("runtime_commit_reason_code_missing")

    runtime_commit_policy_reason_code = report.get("runtime_commit_policy_reason_code")
    if not isinstance(runtime_commit_policy_reason_code, str) or not runtime_commit_policy_reason_code.strip():
        reason_codes.append("runtime_commit_policy_reason_code_missing")

    runtime_commit_nested_reason_code = report.get("runtime_commit_nested_reason_code")
    if not isinstance(runtime_commit_nested_reason_code, str) or not runtime_commit_nested_reason_code.strip():
        reason_codes.append("runtime_commit_nested_reason_code_missing")

    runtime_commit_failure_taxonomy_version = report.get("runtime_commit_failure_taxonomy_version")
    if runtime_commit_failure_taxonomy_version != "v1":
        reason_codes.append("runtime_commit_failure_taxonomy_version_mismatch")

    runtime_commit_failure_taxonomy = report.get("runtime_commit_failure_taxonomy")
    if not isinstance(runtime_commit_failure_taxonomy, str) or not runtime_commit_failure_taxonomy.strip():
        reason_codes.append("runtime_commit_failure_taxonomy_missing")
    elif runtime_commit_failure_taxonomy not in ALLOWED_RUNTIME_COMMIT_FAILURE_TAXONOMIES:
        reason_codes.append("runtime_commit_failure_taxonomy_invalid")

    runtime_commit_failure_diagnostic_hint = report.get("runtime_commit_failure_diagnostic_hint")
    if not isinstance(runtime_commit_failure_diagnostic_hint, str) or not runtime_commit_failure_diagnostic_hint.strip():
        reason_codes.append("runtime_commit_failure_diagnostic_hint_missing")

    runtime_signer_fallback_private_key_env = report.get("runtime_signer_fallback_private_key_env")
    if not isinstance(runtime_signer_fallback_private_key_env, str) or not runtime_signer_fallback_private_key_env.strip():
        reason_codes.append("runtime_signer_fallback_private_key_env_missing")
    elif runtime_signer_fallback_private_key_env != RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV:
        reason_codes.append("runtime_signer_fallback_private_key_env_mismatch")

    runtime_signer_fallback_private_key_present = report.get("runtime_signer_fallback_private_key_present")
    if not isinstance(runtime_signer_fallback_private_key_present, bool):
        reason_codes.append("runtime_signer_fallback_private_key_present_invalid")
    elif runtime_signer_fallback_private_key_present:
        reason_codes.append("runtime_signer_fallback_private_key_present_violation")

    contracts = report.get("contracts")
    if not isinstance(contracts, dict):
        reason_codes.append("contracts_missing")
    else:
        if contracts.get("ci_fast_gate_scope") != "local-only":
            reason_codes.append("ci_fast_gate_scope_mismatch")
        if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
            reason_codes.append("runtime_provider_client_contract_contract_mismatch")
        if contracts.get("runtime_signing_profile") != RUNTIME_REAL_SIGNING_PROFILE_VALUE:
            reason_codes.append("runtime_signing_profile_contract_mismatch")
        if isinstance(runtime_profile, str) and runtime_profile in ("standard", "real-node"):
            if contracts.get("runtime_profile") != runtime_profile:
                reason_codes.append("runtime_profile_contract_mismatch")
        if normalized_runtime_signer_key_source and contracts.get("runtime_signer_key_source") != normalized_runtime_signer_key_source:
            reason_codes.append("runtime_signer_key_source_contract_mismatch")
        if (
            expected_runtime_signer_private_key_env
            and contracts.get("runtime_signer_private_key_env") != expected_runtime_signer_private_key_env
        ):
            reason_codes.append("runtime_signer_private_key_env_contract_mismatch")
        if (
            expected_runtime_signer_key_reference_env
            and contracts.get("runtime_signer_key_reference_env") != expected_runtime_signer_key_reference_env
        ):
            reason_codes.append("runtime_signer_key_reference_env_contract_mismatch")
        if contracts.get("runtime_commit_endpoint") != "/broadcast/runtime-commit":
            reason_codes.append("runtime_commit_endpoint_mismatch")
        if contracts.get("runtime_commit_method") != "POST":
            reason_codes.append("runtime_commit_method_mismatch")
        if contracts.get("runtime_commit_finality_primary_endpoint") != "/notifications":
            reason_codes.append("runtime_commit_finality_primary_endpoint_mismatch")
        if contracts.get("runtime_commit_finality_fallback_endpoint") != "/block/{height}":
            reason_codes.append("runtime_commit_finality_fallback_endpoint_mismatch")
        if contracts.get("runtime_commit_failure_taxonomy_version") != "v1":
            reason_codes.append("runtime_commit_failure_taxonomy_contract_version_mismatch")
        if contracts.get("runtime_signer_fallback_private_key_env") != RUNTIME_SIGNER_FALLBACK_PRIVATE_KEY_ENV:
            reason_codes.append("runtime_signer_fallback_private_key_env_contract_mismatch")
        if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
            reason_codes.append("runtime_signer_fallback_private_key_allowed_contract_mismatch")
        if contracts.get("runtime_signer_managed_external_raw_private_key_allowed") is not False:
            reason_codes.append("runtime_signer_managed_external_raw_private_key_allowed_contract_mismatch")
        if contracts.get("runtime_signer_attestation_schema_version") != RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION:
            reason_codes.append("runtime_signer_attestation_schema_version_contract_mismatch")
        if contracts.get("runtime_signer_attestation_signer_uniqueness_required") is not True:
            reason_codes.append("runtime_signer_attestation_signer_uniqueness_required_contract_mismatch")
        if contracts.get("runtime_signer_attestation_threshold_required") is not True:
            reason_codes.append("runtime_signer_attestation_threshold_required_contract_mismatch")
        if contracts.get("runtime_signer_attestation_profile_membership_required") is not True:
            reason_codes.append("runtime_signer_attestation_profile_membership_required_contract_mismatch")
        if contracts.get("runtime_signer_attestation_required_approvals") != 1:
            reason_codes.append("runtime_signer_attestation_required_approvals_contract_mismatch")

    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "bootstrap_readiness",
            "localhost_signed_integration",
            "live_api_conformance",
            "runtime_signer_fallback_private_key_contract",
            "runtime_signer_managed_external_raw_private_key_contract",
            "runtime_commit_endpoint",
            "runtime_commit_policy",
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

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not all(
        isinstance(path, str) and path.strip() for path in artifact_paths
    ):
        reason_codes.append("artifact_paths_invalid")
    elif isinstance(runtime_commit_live_policy_report, str):
        if runtime_commit_live_policy_report not in artifact_paths:
            reason_codes.append("runtime_commit_live_policy_report_artifact_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "live_runtime_integration_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if mode == "run" and runtime_commit_policy_reason_code != "runtime_commit_policy_passed":
            reason_codes.append("runtime_commit_policy_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "live_runtime_integration_passed"):
            reason_codes.append("fail_status_reason_code_mismatch")

    expected_failure_taxonomy = expected_runtime_commit_failure_taxonomy(
        status,
        reason_code,
        runtime_commit_reason_code,
        runtime_commit_policy_reason_code,
        runtime_commit_nested_reason_code,
    )
    if (
        isinstance(runtime_commit_failure_taxonomy, str)
        and runtime_commit_failure_taxonomy.strip()
        and runtime_commit_failure_taxonomy in ALLOWED_RUNTIME_COMMIT_FAILURE_TAXONOMIES
        and runtime_commit_failure_taxonomy != expected_failure_taxonomy
    ):
        reason_codes.append(f"runtime_commit_failure_taxonomy_mismatch:{expected_failure_taxonomy}")

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
        "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1",
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
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
