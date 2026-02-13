#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

NON_SYNTHETIC_SUBMIT_PROBE_MARKER = "integration_kolme_fork_live_node_submit_reaches_endpoint"
IN_MEMORY_PROVIDER_MARKER = "InMemoryKolmeRuntimeCommitClient"
REAL_SIGNING_PROFILE_ENV = "KAMN_KOLME_LIVE_SIGNING_PROFILE"
REAL_SIGNING_PROFILE_VALUE = "kolme-fork-secp256k1-v1"
REAL_SIGNING_PROFILE_MARKER = f"{REAL_SIGNING_PROFILE_ENV}={REAL_SIGNING_PROFILE_VALUE}"
REAL_SIGNING_PROFILE_PATTERN = re.compile(rf"{REAL_SIGNING_PROFILE_ENV}=([^\s\"']+)")
REAL_SIGNER_PROFILE_SELECTOR_ENV = "KAMN_KOLME_LIVE_SIGNER_PROFILE"
REAL_SIGNER_PROFILE_PRIMARY = "ops-primary"
REAL_SIGNER_PROFILE_SECONDARY = "ops-secondary"
REAL_SIGNER_PROFILE_SELECTOR_PATTERN = re.compile(
    rf"{REAL_SIGNER_PROFILE_SELECTOR_ENV}=([^\s\"']+)"
)
REAL_SIGNER_PRIVATE_KEY_ENV_PRIMARY = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
REAL_SIGNER_PRIVATE_KEY_ENV_SECONDARY = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
REAL_SIGNER_KEY_REF_ENV_PRIMARY = "KAMN_KOLME_LIVE_SIGNER_KEY_REF"
REAL_SIGNER_KEY_REF_ENV_SECONDARY = "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY"
REAL_SIGNER_PUBLIC_KEY_ENV_PRIMARY = "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"
REAL_SIGNER_PUBLIC_KEY_ENV_SECONDARY = "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"
REAL_SIGNER_FALLBACK_PRIVATE_KEY_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
REAL_SIGNER_FALLBACK_GUARD_CONTRACT_VERSION = "v2"
REAL_SIGNER_FALLBACK_GUARD_MODE = "reject_if_present"
REAL_SIGNER_KEY_SOURCE_CONTRACT_VERSION = "v1"
ALLOWED_SIGNER_PROFILES = (
    REAL_SIGNER_PROFILE_PRIMARY,
    REAL_SIGNER_PROFILE_SECONDARY,
)
ALLOWED_SIGNER_KEY_SOURCES_BY_PROFILE = {
    REAL_SIGNER_PROFILE_PRIMARY: ("env-local", "managed-external"),
    REAL_SIGNER_PROFILE_SECONDARY: ("env-local",),
}
ALLOWED_SIGNER_KEY_SOURCES = ("env-local", "managed-external")
SIGNER_PRIVATE_KEY_ENV_BY_PROFILE = {
    REAL_SIGNER_PROFILE_PRIMARY: REAL_SIGNER_PRIVATE_KEY_ENV_PRIMARY,
    REAL_SIGNER_PROFILE_SECONDARY: REAL_SIGNER_PRIVATE_KEY_ENV_SECONDARY,
}
SIGNER_KEY_REF_ENV_BY_PROFILE = {
    REAL_SIGNER_PROFILE_PRIMARY: REAL_SIGNER_KEY_REF_ENV_PRIMARY,
    REAL_SIGNER_PROFILE_SECONDARY: REAL_SIGNER_KEY_REF_ENV_SECONDARY,
}
SIGNER_PUBLIC_KEY_ENV_BY_PROFILE = {
    REAL_SIGNER_PROFILE_PRIMARY: REAL_SIGNER_PUBLIC_KEY_ENV_PRIMARY,
    REAL_SIGNER_PROFILE_SECONDARY: REAL_SIGNER_PUBLIC_KEY_ENV_SECONDARY,
}
NATIVE_PAYLOAD_PUBKEY_MARKER = "pubkey"
NATIVE_PAYLOAD_NONCE_MARKER = "nonce"
NATIVE_PAYLOAD_MESSAGES_MARKER = "messages"
RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION = "kamn.kolme.runtime-signer-attestation.v1"
RUNTIME_SIGNER_FAILOVER_ATTESTATION_MIN_REQUIRED_APPROVALS = 2


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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local KAMN live runtime integration real-node profile summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument(
        "--require-non-synthetic-run-evidence",
        action="store_true",
        help="Require strict non-synthetic runtime evidence marker propagation.",
    )
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
    elif ci_fast_gate_eligible:
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

    runtime_profile = report.get("runtime_profile")
    if not isinstance(runtime_profile, str) or not runtime_profile.strip():
        reason_codes.append("runtime_profile_missing")
    elif runtime_profile != "real-node":
        reason_codes.append("runtime_profile_mismatch")

    runtime_signing_profile = report.get("runtime_signing_profile")
    if not isinstance(runtime_signing_profile, str) or not runtime_signing_profile.strip():
        reason_codes.append("runtime_signing_profile_missing")
    elif runtime_signing_profile != REAL_SIGNING_PROFILE_VALUE:
        reason_codes.append("runtime_signing_profile_mismatch")

    runtime_provider_client_contract = report.get("runtime_provider_client_contract")
    if not isinstance(runtime_provider_client_contract, str) or not runtime_provider_client_contract.strip():
        reason_codes.append("runtime_provider_client_contract_missing")
    elif runtime_provider_client_contract != "KolmeRuntimeCommitLiveProvider":
        reason_codes.append("runtime_provider_client_contract_mismatch")

    runtime_signer_profile_selector_env = report.get("runtime_signer_profile_selector_env")
    if not isinstance(runtime_signer_profile_selector_env, str) or not runtime_signer_profile_selector_env.strip():
        reason_codes.append("runtime_signer_profile_selector_env_missing")
    elif runtime_signer_profile_selector_env != REAL_SIGNER_PROFILE_SELECTOR_ENV:
        reason_codes.append("runtime_signer_profile_selector_env_mismatch")

    runtime_signer_profile = report.get("runtime_signer_profile")
    expected_signer_private_key_env = ""
    expected_signer_key_reference_env = ""
    expected_signer_public_key_env = ""
    if not isinstance(runtime_signer_profile, str) or not runtime_signer_profile.strip():
        reason_codes.append("runtime_signer_profile_missing")
    elif runtime_signer_profile not in ALLOWED_SIGNER_PROFILES:
        reason_codes.append("runtime_signer_profile_mismatch")
    else:
        expected_signer_private_key_env = SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[runtime_signer_profile]
        expected_signer_key_reference_env = SIGNER_KEY_REF_ENV_BY_PROFILE[runtime_signer_profile]
        expected_signer_public_key_env = SIGNER_PUBLIC_KEY_ENV_BY_PROFILE[runtime_signer_profile]

    runtime_signer_previous_profile = report.get("runtime_signer_previous_profile")
    if not isinstance(runtime_signer_previous_profile, str) or not runtime_signer_previous_profile.strip():
        reason_codes.append("runtime_signer_previous_profile_missing")
    elif runtime_signer_previous_profile not in ALLOWED_SIGNER_PROFILES:
        reason_codes.append("runtime_signer_previous_profile_mismatch")

    runtime_signer_failover_active = report.get("runtime_signer_failover_active")
    if not isinstance(runtime_signer_failover_active, bool):
        reason_codes.append("runtime_signer_failover_active_invalid")

    runtime_signer_rotation_epoch = report.get("runtime_signer_rotation_epoch")
    if not isinstance(runtime_signer_rotation_epoch, int) or runtime_signer_rotation_epoch <= 0:
        reason_codes.append("runtime_signer_rotation_epoch_invalid")

    runtime_signer_previous_rotation_epoch = report.get("runtime_signer_previous_rotation_epoch")
    if not isinstance(runtime_signer_previous_rotation_epoch, int) or runtime_signer_previous_rotation_epoch <= 0:
        reason_codes.append("runtime_signer_previous_rotation_epoch_invalid")

    if (
        isinstance(runtime_signer_profile, str)
        and runtime_signer_profile in ALLOWED_SIGNER_PROFILES
        and isinstance(runtime_signer_previous_profile, str)
        and runtime_signer_previous_profile in ALLOWED_SIGNER_PROFILES
        and isinstance(runtime_signer_failover_active, bool)
    ):
        if runtime_signer_failover_active and runtime_signer_profile == runtime_signer_previous_profile:
            reason_codes.append("runtime_signer_failover_profile_unchanged")
        if (not runtime_signer_failover_active) and runtime_signer_profile != runtime_signer_previous_profile:
            reason_codes.append("runtime_signer_profile_changed_without_failover")
    if (
        isinstance(runtime_signer_failover_active, bool)
        and runtime_signer_failover_active
        and isinstance(runtime_signer_rotation_epoch, int)
        and isinstance(runtime_signer_previous_rotation_epoch, int)
        and runtime_signer_rotation_epoch <= runtime_signer_previous_rotation_epoch
    ):
        reason_codes.append("runtime_signer_rotation_epoch_stale")

    runtime_signer_key_source_contract_version = report.get("runtime_signer_key_source_contract_version")
    if not isinstance(runtime_signer_key_source_contract_version, str) or not runtime_signer_key_source_contract_version.strip():
        reason_codes.append("runtime_signer_key_source_contract_version_missing")
    elif runtime_signer_key_source_contract_version != REAL_SIGNER_KEY_SOURCE_CONTRACT_VERSION:
        reason_codes.append("runtime_signer_key_source_contract_version_mismatch")

    runtime_signer_key_source = report.get("runtime_signer_key_source")
    expected_signer_key_source = ""
    if not isinstance(runtime_signer_key_source, str) or not runtime_signer_key_source.strip():
        reason_codes.append("runtime_signer_key_source_missing")
    else:
        expected_signer_key_source = runtime_signer_key_source.strip()
        if expected_signer_key_source not in ALLOWED_SIGNER_KEY_SOURCES:
            reason_codes.append("runtime_signer_key_source_invalid")
    if (
        isinstance(runtime_signer_profile, str)
        and runtime_signer_profile in ALLOWED_SIGNER_PROFILES
        and expected_signer_key_source
        and expected_signer_key_source in ALLOWED_SIGNER_KEY_SOURCES
        and expected_signer_key_source not in ALLOWED_SIGNER_KEY_SOURCES_BY_PROFILE[runtime_signer_profile]
    ):
        reason_codes.append("runtime_signer_key_source_profile_pair_disallowed")

    runtime_signer_private_key_env = report.get("runtime_signer_private_key_env")
    if not isinstance(runtime_signer_private_key_env, str) or not runtime_signer_private_key_env.strip():
        reason_codes.append("runtime_signer_private_key_env_missing")
    elif expected_signer_private_key_env and runtime_signer_private_key_env != expected_signer_private_key_env:
        reason_codes.append("runtime_signer_private_key_env_mismatch")

    runtime_signer_key_reference_env = report.get("runtime_signer_key_reference_env")
    if not isinstance(runtime_signer_key_reference_env, str) or not runtime_signer_key_reference_env.strip():
        reason_codes.append("runtime_signer_key_reference_env_missing")
    elif expected_signer_key_reference_env and runtime_signer_key_reference_env != expected_signer_key_reference_env:
        reason_codes.append("runtime_signer_key_reference_env_mismatch")

    runtime_signer_fallback_guard_contract_version = report.get(
        "runtime_signer_fallback_guard_contract_version"
    )
    if (
        not isinstance(runtime_signer_fallback_guard_contract_version, str)
        or not runtime_signer_fallback_guard_contract_version.strip()
    ):
        reason_codes.append("runtime_signer_fallback_guard_contract_version_missing")
    elif (
        runtime_signer_fallback_guard_contract_version
        != REAL_SIGNER_FALLBACK_GUARD_CONTRACT_VERSION
    ):
        reason_codes.append("runtime_signer_fallback_guard_contract_version_mismatch")

    runtime_signer_fallback_guard_mode = report.get(
        "runtime_signer_fallback_guard_mode"
    )
    if (
        not isinstance(runtime_signer_fallback_guard_mode, str)
        or not runtime_signer_fallback_guard_mode.strip()
    ):
        reason_codes.append("runtime_signer_fallback_guard_mode_missing")
    elif runtime_signer_fallback_guard_mode != REAL_SIGNER_FALLBACK_GUARD_MODE:
        reason_codes.append("runtime_signer_fallback_guard_mode_mismatch")

    expected_managed_external_raw_private_key_remediation = ""
    if expected_signer_private_key_env and expected_signer_key_reference_env:
        expected_managed_external_raw_private_key_remediation = (
            f"unset {expected_signer_private_key_env}; set {expected_signer_key_reference_env}"
        )

    runtime_signer_managed_external_raw_private_key_remediation = report.get(
        "runtime_signer_managed_external_raw_private_key_remediation"
    )
    if (
        not isinstance(runtime_signer_managed_external_raw_private_key_remediation, str)
        or not runtime_signer_managed_external_raw_private_key_remediation.strip()
    ):
        reason_codes.append("runtime_signer_managed_external_raw_private_key_remediation_missing")
    elif (
        expected_managed_external_raw_private_key_remediation
        and runtime_signer_managed_external_raw_private_key_remediation
        != expected_managed_external_raw_private_key_remediation
    ):
        reason_codes.append("runtime_signer_managed_external_raw_private_key_remediation_mismatch")

    runtime_signer_fallback_private_key_present = report.get("runtime_signer_fallback_private_key_present")
    if not isinstance(runtime_signer_fallback_private_key_present, bool):
        reason_codes.append("runtime_signer_fallback_private_key_present_invalid")
    elif runtime_signer_fallback_private_key_present:
        reason_codes.append("runtime_signer_fallback_private_key_present_violation")

    runtime_signer_raw_private_key_present = report.get("runtime_signer_raw_private_key_present")
    if not isinstance(runtime_signer_raw_private_key_present, bool):
        reason_codes.append("runtime_signer_raw_private_key_present_invalid")
    elif expected_signer_key_source == "managed-external" and runtime_signer_raw_private_key_present:
        reason_codes.append("runtime_signer_managed_external_raw_private_key_present_violation")

    runtime_signer_attestation_schema_version = report.get("runtime_signer_attestation_schema_version")
    if (
        not isinstance(runtime_signer_attestation_schema_version, str)
        or not runtime_signer_attestation_schema_version.strip()
    ):
        reason_codes.append("runtime_signer_attestation_schema_version_missing")
    elif runtime_signer_attestation_schema_version != RUNTIME_SIGNER_ATTESTATION_SCHEMA_VERSION:
        reason_codes.append("runtime_signer_attestation_schema_version_mismatch")

    runtime_signer_attestation_bundle = report.get("runtime_signer_attestation_bundle")
    reason_codes.extend(
        evaluate_runtime_signer_attestation_bundle(
            runtime_signer_attestation_bundle,
            runtime_signer_profile,
        )
    )

    runtime_signer_attestation_required_approvals: int | None = None
    runtime_signer_attestation_approved_signers: list[str] = []
    if isinstance(runtime_signer_attestation_bundle, dict):
        required_approvals_value = runtime_signer_attestation_bundle.get("required_approvals")
        if isinstance(required_approvals_value, int):
            runtime_signer_attestation_required_approvals = required_approvals_value

        approved_signers_value = runtime_signer_attestation_bundle.get("approved_signers")
        if isinstance(approved_signers_value, list):
            for entry in approved_signers_value:
                if isinstance(entry, str) and entry.strip():
                    runtime_signer_attestation_approved_signers.append(entry.strip())

    runtime_signer_quorum_linkage_contract_version = report.get(
        "runtime_signer_quorum_linkage_contract_version"
    )
    if runtime_signer_quorum_linkage_contract_version is not None:
        if (
            not isinstance(runtime_signer_quorum_linkage_contract_version, str)
            or not runtime_signer_quorum_linkage_contract_version.strip()
        ):
            reason_codes.append("runtime_signer_quorum_linkage_contract_version_invalid")
        elif runtime_signer_quorum_linkage_contract_version != "v1":
            reason_codes.append("runtime_signer_quorum_linkage_contract_version_mismatch")

    runtime_signer_quorum_required_approvals = report.get(
        "runtime_signer_quorum_required_approvals"
    )
    if runtime_signer_quorum_required_approvals is not None:
        if (
            not isinstance(runtime_signer_quorum_required_approvals, int)
            or runtime_signer_quorum_required_approvals <= 0
        ):
            reason_codes.append("runtime_signer_quorum_required_approvals_invalid")
        elif (
            isinstance(runtime_signer_attestation_required_approvals, int)
            and runtime_signer_quorum_required_approvals
            != runtime_signer_attestation_required_approvals
        ):
            reason_codes.append("runtime_signer_quorum_required_approvals_mismatch")

    expected_runtime_signer_quorum_profile_linked = (
        isinstance(runtime_signer_profile, str)
        and runtime_signer_profile.strip()
        and runtime_signer_profile in runtime_signer_attestation_approved_signers
    )
    expected_runtime_signer_quorum_satisfied = (
        isinstance(runtime_signer_attestation_required_approvals, int)
        and len(runtime_signer_attestation_approved_signers)
        >= runtime_signer_attestation_required_approvals
    )
    expected_runtime_signer_quorum_linked = (
        expected_runtime_signer_quorum_profile_linked
        and expected_runtime_signer_quorum_satisfied
    )

    runtime_signer_quorum_approved_signers_count = report.get(
        "runtime_signer_quorum_approved_signers_count"
    )
    if runtime_signer_quorum_approved_signers_count is not None:
        if (
            not isinstance(runtime_signer_quorum_approved_signers_count, int)
            or runtime_signer_quorum_approved_signers_count < 0
        ):
            reason_codes.append("runtime_signer_quorum_approved_signers_count_invalid")
        elif runtime_signer_quorum_approved_signers_count != len(
            runtime_signer_attestation_approved_signers
        ):
            reason_codes.append("runtime_signer_quorum_approved_signers_count_mismatch")

    runtime_signer_quorum_profile_linked = report.get("runtime_signer_quorum_profile_linked")
    if runtime_signer_quorum_profile_linked is not None:
        if not isinstance(runtime_signer_quorum_profile_linked, bool):
            reason_codes.append("runtime_signer_quorum_profile_linked_invalid")
        elif runtime_signer_quorum_profile_linked != bool(
            expected_runtime_signer_quorum_profile_linked
        ):
            reason_codes.append("runtime_signer_quorum_profile_linked_mismatch")

    runtime_signer_quorum_satisfied = report.get("runtime_signer_quorum_satisfied")
    if runtime_signer_quorum_satisfied is not None:
        if not isinstance(runtime_signer_quorum_satisfied, bool):
            reason_codes.append("runtime_signer_quorum_satisfied_invalid")
        elif runtime_signer_quorum_satisfied != bool(expected_runtime_signer_quorum_satisfied):
            reason_codes.append("runtime_signer_quorum_satisfied_mismatch")

    runtime_signer_quorum_linked = report.get("runtime_signer_quorum_linked")
    if runtime_signer_quorum_linked is None:
        runtime_signer_quorum_linked = expected_runtime_signer_quorum_linked
    elif not isinstance(runtime_signer_quorum_linked, bool):
        reason_codes.append("runtime_signer_quorum_linked_invalid")
    elif runtime_signer_quorum_linked != bool(expected_runtime_signer_quorum_linked):
        reason_codes.append("runtime_signer_quorum_linkage_drift")

    if runtime_signer_quorum_linked is False:
        reason_codes.append("runtime_signer_quorum_linkage_violation")

    if isinstance(runtime_signer_failover_active, bool) and runtime_signer_failover_active:
        if (
            not isinstance(runtime_signer_attestation_required_approvals, int)
            or runtime_signer_attestation_required_approvals
            < RUNTIME_SIGNER_FAILOVER_ATTESTATION_MIN_REQUIRED_APPROVALS
        ):
            reason_codes.append("runtime_signer_failover_attestation_required_approvals_insufficient")
        if (
            isinstance(runtime_signer_previous_profile, str)
            and runtime_signer_previous_profile.strip()
            and runtime_signer_previous_profile not in runtime_signer_attestation_approved_signers
        ):
            reason_codes.append("runtime_signer_failover_attestation_previous_profile_not_approved")

    runtime_commit_command_profile = report.get("runtime_commit_command_profile")
    if not isinstance(runtime_commit_command_profile, str) or not runtime_commit_command_profile.strip():
        reason_codes.append("runtime_commit_command_profile_missing")
    elif runtime_commit_command_profile != "real-node-non-synthetic-v1":
        reason_codes.append("runtime_commit_command_profile_mismatch")

    runtime_commit_policy_command_profile = report.get("runtime_commit_policy_command_profile")
    if not isinstance(runtime_commit_policy_command_profile, str) or not runtime_commit_policy_command_profile.strip():
        reason_codes.append("runtime_commit_policy_command_profile_missing")
    elif runtime_commit_policy_command_profile != "real-node-non-synthetic-v1":
        reason_codes.append("runtime_commit_policy_command_profile_mismatch")

    runtime_commit_command_profile_version = report.get("runtime_commit_command_profile_version")
    if not isinstance(runtime_commit_command_profile_version, str) or not runtime_commit_command_profile_version.strip():
        reason_codes.append("runtime_commit_command_profile_version_missing")
    elif runtime_commit_command_profile_version != "v1":
        reason_codes.append("runtime_commit_command_profile_version_mismatch")

    runtime_commit_command = report.get("runtime_commit_command")
    if not isinstance(runtime_commit_command, str) or not runtime_commit_command.strip():
        reason_codes.append("runtime_commit_command_missing")
    else:
        observed_signing_profile_values = [
            observed_value.rstrip("\\")
            for observed_value in REAL_SIGNING_PROFILE_PATTERN.findall(runtime_commit_command)
        ]
        observed_signer_profile_values = [
            observed_value.rstrip("\\")
            for observed_value in REAL_SIGNER_PROFILE_SELECTOR_PATTERN.findall(runtime_commit_command)
        ]
        if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in runtime_commit_command:
            reason_codes.append("runtime_commit_contract_lane_missing")
        if "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider" not in runtime_commit_command:
            reason_codes.append("runtime_provider_contract_marker_missing")
        if args.require_non_synthetic_run_evidence and "--require-non-synthetic-run-evidence" not in runtime_commit_command:
            reason_codes.append("runtime_commit_non_synthetic_policy_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and NON_SYNTHETIC_SUBMIT_PROBE_MARKER not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_non_synthetic_submit_probe_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and REAL_SIGNING_PROFILE_MARKER not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_real_signing_profile_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and any(
                observed_signing_profile and observed_signing_profile != REAL_SIGNING_PROFILE_VALUE
                for observed_signing_profile in observed_signing_profile_values
            )
        ):
            reason_codes.append("runtime_commit_signing_profile_value_disallowed")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and any("simulated" in observed_signing_profile.lower() for observed_signing_profile in observed_signing_profile_values)
        ):
            reason_codes.append("runtime_commit_simulated_signing_profile_detected")
        expected_profile_command_marker = ""
        if isinstance(runtime_signer_profile, str) and runtime_signer_profile in ALLOWED_SIGNER_PROFILES:
            expected_profile_command_marker = (
                f"{REAL_SIGNER_PROFILE_SELECTOR_ENV}={runtime_signer_profile}"
            )
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and expected_profile_command_marker
            and expected_profile_command_marker not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_signer_profile_marker_missing")
        if runtime_commit_command_profile == "real-node-non-synthetic-v1":
            observed_allowed_signer_profiles = {
                observed_profile
                for observed_profile in observed_signer_profile_values
                if observed_profile in ALLOWED_SIGNER_PROFILES
            }
            if len(observed_allowed_signer_profiles) > 1:
                reason_codes.append("runtime_commit_signer_profile_split_brain_detected")
        expected_key_source_command_marker = ""
        if expected_signer_key_source:
            expected_key_source_command_marker = (
                f"KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE={expected_signer_key_source}"
            )
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and expected_key_source_command_marker
            and expected_key_source_command_marker not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_signer_key_source_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and expected_signer_key_source == "managed-external"
            and expected_signer_key_reference_env
            and f"{expected_signer_key_reference_env}=" not in runtime_commit_command
        ):
            reason_codes.append(
                "runtime_commit_managed_external_signer_key_reference_marker_missing"
            )
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and expected_signer_key_source == "managed-external"
            and expected_signer_public_key_env
            and f"{expected_signer_public_key_env}=" not in runtime_commit_command
        ):
            reason_codes.append(
                "runtime_commit_managed_external_signer_public_key_marker_missing"
            )
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and expected_signer_key_source == "managed-external"
            and expected_signer_private_key_env
            and f"{expected_signer_private_key_env}=" in runtime_commit_command
        ):
            reason_codes.append(
                "runtime_commit_managed_external_private_key_command_marker_detected"
            )
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and f"{REAL_SIGNER_FALLBACK_PRIVATE_KEY_ENV}=" in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_fallback_private_key_command_marker_detected")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and NATIVE_PAYLOAD_PUBKEY_MARKER not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_native_payload_pubkey_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and NATIVE_PAYLOAD_NONCE_MARKER not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_native_payload_nonce_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and NATIVE_PAYLOAD_MESSAGES_MARKER not in runtime_commit_command
        ):
            reason_codes.append("runtime_commit_native_payload_messages_marker_missing")
        if IN_MEMORY_PROVIDER_MARKER in runtime_commit_command:
            reason_codes.append("runtime_commit_in_memory_provider_reference_detected")

    runtime_commit_live_policy_report = report.get("runtime_commit_live_policy_report")
    if not isinstance(runtime_commit_live_policy_report, str) or not runtime_commit_live_policy_report.strip():
        reason_codes.append("runtime_commit_live_policy_report_missing")

    contracts = report.get("contracts")
    if not isinstance(contracts, dict):
        reason_codes.append("contracts_missing")
    else:
        if contracts.get("ci_fast_gate_scope") != "local-only":
            reason_codes.append("ci_fast_gate_scope_mismatch")
        if contracts.get("runtime_profile") != "real-node":
            reason_codes.append("runtime_profile_contract_mismatch")
        if contracts.get("runtime_signing_profile") != REAL_SIGNING_PROFILE_VALUE:
            reason_codes.append("runtime_signing_profile_contract_mismatch")
        if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
            reason_codes.append("runtime_provider_client_contract_contract_mismatch")
        if contracts.get("runtime_signer_profile_selector_env") != REAL_SIGNER_PROFILE_SELECTOR_ENV:
            reason_codes.append("runtime_signer_profile_selector_env_contract_mismatch")
        if (
            isinstance(runtime_signer_profile, str)
            and runtime_signer_profile in ALLOWED_SIGNER_PROFILES
            and contracts.get("runtime_signer_profile") != runtime_signer_profile
        ):
            reason_codes.append("runtime_signer_profile_contract_mismatch")
        if (
            isinstance(runtime_signer_key_source_contract_version, str)
            and runtime_signer_key_source_contract_version == REAL_SIGNER_KEY_SOURCE_CONTRACT_VERSION
            and contracts.get("runtime_signer_key_source_contract_version") != runtime_signer_key_source_contract_version
        ):
            reason_codes.append("runtime_signer_key_source_contract_version_contract_mismatch")
        if expected_signer_key_source and contracts.get("runtime_signer_key_source") != expected_signer_key_source:
            reason_codes.append("runtime_signer_key_source_contract_mismatch")
        if expected_signer_private_key_env and contracts.get("runtime_signer_private_key_env") != expected_signer_private_key_env:
            reason_codes.append("runtime_signer_private_key_env_contract_mismatch")
        if expected_signer_key_reference_env and contracts.get("runtime_signer_key_reference_env") != expected_signer_key_reference_env:
            reason_codes.append("runtime_signer_key_reference_env_contract_mismatch")
        if (
            contracts.get("runtime_signer_fallback_guard_contract_version")
            != REAL_SIGNER_FALLBACK_GUARD_CONTRACT_VERSION
        ):
            reason_codes.append(
                "runtime_signer_fallback_guard_contract_version_contract_mismatch"
            )
        if (
            contracts.get("runtime_signer_fallback_guard_mode")
            != REAL_SIGNER_FALLBACK_GUARD_MODE
        ):
            reason_codes.append("runtime_signer_fallback_guard_mode_contract_mismatch")
        if (
            expected_managed_external_raw_private_key_remediation
            and contracts.get("runtime_signer_managed_external_raw_private_key_remediation")
            != expected_managed_external_raw_private_key_remediation
        ):
            reason_codes.append(
                "runtime_signer_managed_external_raw_private_key_remediation_contract_mismatch"
            )
        if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
            reason_codes.append("runtime_signer_fallback_private_key_allowed_contract_mismatch")
        if contracts.get("runtime_signer_fallback_private_key_command_marker_allowed") is not False:
            reason_codes.append("runtime_signer_fallback_private_key_command_marker_allowed_contract_mismatch")
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
        if contracts.get("runtime_signer_quorum_linkage_contract_version") not in (None, "v1"):
            reason_codes.append("runtime_signer_quorum_linkage_contract_version_contract_mismatch")
        expected_runtime_signer_quorum_required_approvals = (
            runtime_signer_attestation_required_approvals
            if isinstance(runtime_signer_attestation_required_approvals, int)
            else 1
        )
        if contracts.get("runtime_signer_quorum_required_approvals") not in (
            None,
            expected_runtime_signer_quorum_required_approvals,
        ):
            reason_codes.append("runtime_signer_quorum_required_approvals_contract_mismatch")
        if contracts.get("runtime_signer_quorum_linked_required") not in (None, True):
            reason_codes.append("runtime_signer_quorum_linked_required_contract_mismatch")
        if contracts.get("runtime_signer_quorum_threshold_required") not in (None, True):
            reason_codes.append("runtime_signer_quorum_threshold_required_contract_mismatch")
        if contracts.get("runtime_signer_quorum_profile_membership_required") not in (
            None,
            True,
        ):
            reason_codes.append(
                "runtime_signer_quorum_profile_membership_required_contract_mismatch"
            )
        if contracts.get("runtime_signer_quorum_linked") not in (
            None,
            bool(expected_runtime_signer_quorum_linked),
        ):
            reason_codes.append("runtime_signer_quorum_linked_contract_mismatch")
        expected_runtime_signer_attestation_required_approvals = (
            RUNTIME_SIGNER_FAILOVER_ATTESTATION_MIN_REQUIRED_APPROVALS
            if runtime_signer_failover_active is True
            else 1
        )
        if (
            contracts.get("runtime_signer_attestation_required_approvals")
            != expected_runtime_signer_attestation_required_approvals
        ):
            reason_codes.append("runtime_signer_attestation_required_approvals_contract_mismatch")
        if (
            contracts.get("runtime_signer_failover_attestation_min_required_approvals")
            != RUNTIME_SIGNER_FAILOVER_ATTESTATION_MIN_REQUIRED_APPROVALS
        ):
            reason_codes.append(
                "runtime_signer_failover_attestation_min_required_approvals_contract_mismatch"
            )
        if (
            contracts.get("runtime_signer_failover_attestation_previous_profile_membership_required")
            is not True
        ):
            reason_codes.append(
                "runtime_signer_failover_attestation_previous_profile_membership_contract_mismatch"
            )
        if contracts.get("runtime_signer_failover_requires_profile_change") is not True:
            reason_codes.append("runtime_signer_failover_requires_profile_change_contract_mismatch")
        if contracts.get("runtime_signer_rotation_epoch_must_increase_on_failover") is not True:
            reason_codes.append("runtime_signer_rotation_epoch_contract_mismatch")
        if contracts.get("runtime_commit_endpoint") != "/broadcast/runtime-commit":
            reason_codes.append("runtime_commit_endpoint_mismatch")
        if contracts.get("runtime_commit_method") != "POST":
            reason_codes.append("runtime_commit_method_mismatch")
        if contracts.get("runtime_commit_finality_primary_endpoint") != "/notifications":
            reason_codes.append("runtime_commit_finality_primary_endpoint_mismatch")
        if contracts.get("runtime_commit_finality_fallback_endpoint") != "/block/{height}":
            reason_codes.append("runtime_commit_finality_fallback_endpoint_mismatch")

    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "bootstrap_readiness",
            "localhost_signed_integration",
            "live_api_conformance",
            "runtime_signer_fallback_private_key_contract",
            "runtime_commit_endpoint",
            "runtime_commit_policy",
        }
        observed_ids: set[str] = set()
        runtime_commit_policy_check_command: str | None = None
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
            elif check_id == "runtime_commit_policy":
                runtime_commit_policy_check_command = command
            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            if not isinstance(check_reason_code, str) or not check_reason_code.strip():
                reason_codes.append(f"check_reason_code_invalid:{check_id}")
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")
        if (
            args.require_non_synthetic_run_evidence
            and runtime_commit_policy_check_command is not None
            and "--require-non-synthetic-run-evidence" not in runtime_commit_policy_check_command
        ):
            reason_codes.append("runtime_commit_policy_check_non_synthetic_marker_missing")
        if (
            runtime_commit_command_profile == "real-node-non-synthetic-v1"
            and runtime_commit_policy_check_command is not None
            and "--require-native-payload-evidence" not in runtime_commit_policy_check_command
        ):
            reason_codes.append("runtime_commit_policy_check_native_payload_marker_missing")
        if runtime_commit_policy_check_command is not None and IN_MEMORY_PROVIDER_MARKER in runtime_commit_policy_check_command:
            reason_codes.append("runtime_commit_policy_check_in_memory_provider_reference_detected")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
    elif (
        isinstance(runtime_commit_live_policy_report, str)
        and runtime_commit_live_policy_report.strip()
        and runtime_commit_live_policy_report not in artifact_paths
    ):
        reason_codes.append("runtime_commit_live_policy_report_artifact_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "live_runtime_integration_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "live_runtime_integration_passed"):
            reason_codes.append("fail_status_reason_code_mismatch")

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
        "schema_version": "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1",
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_reason_codes": args.require_reason_code,
        "require_non_synthetic_run_evidence": args.require_non_synthetic_run_evidence,
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
