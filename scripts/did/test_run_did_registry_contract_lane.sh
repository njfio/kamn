#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_did_registry_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/did/did_registry_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/did_did_registry_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected did registry contract lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected did registry shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$SCRIPT" >"$TMP_OUT"
if ! grep -q "did registry contract lane tests passed." "$TMP_OUT"; then
  echo "expected did registry contract lane success marker" >&2
  exit 1
fi

if ! grep -q "retry_classification_is_deterministic_for_duplicate_submission" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include retry classification test coverage" >&2
  exit 1
fi

if ! grep -q "functional_chain_submission_adapter_returns_typed_submitted_outcome" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include chain adapter submitted outcome coverage" >&2
  exit 1
fi

if ! grep -q "integration_chain_submission_adapter_deduplicates_retry_outcomes" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include chain adapter duplicate outcome integration coverage" >&2
  exit 1
fi

if ! grep -q "regression_chain_submission_adapter_exposes_rejected_outcome_without_panicking" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include chain adapter rejected-outcome regression coverage" >&2
  exit 1
fi

if ! grep -q "integration_register_retry_and_finality_boundary_is_idempotent" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include finality idempotency integration coverage" >&2
  exit 1
fi

if ! grep -q "regression_register_finality_rejects_stale_or_conflicting_updates" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include stale/conflict regression coverage" >&2
  exit 1
fi

if ! grep -q "unit_lifecycle_mutation_nonce_guards_emit_deterministic_reason_codes" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include DID lifecycle mutation nonce unit coverage" >&2
  exit 1
fi

if ! grep -q "functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include DID lifecycle rotate mutation functional coverage" >&2
  exit 1
fi

if ! grep -q "integration_lifecycle_revoke_then_recover_restores_active_resolution" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include DID lifecycle revoke/recover integration coverage" >&2
  exit 1
fi

if ! grep -q "regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include DID lifecycle replay/authorization regression coverage" >&2
  exit 1
fi

if ! grep -q "performance_lifecycle_mutation_contract_lane_stays_within_budget" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include DID lifecycle mutation performance budget coverage" >&2
  exit 1
fi

if ! grep -q "key_lifecycle_audit_trails_docs" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include key lifecycle audit doc contract coverage" >&2
  exit 1
fi

if ! grep -q "run_lifecycle_operator_binding_contract_lane.sh --skip-tests" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to invoke lifecycle operator-binding contract lane coverage" >&2
  exit 1
fi

if ! grep -q "run_service_endpoint_canonicalization_contract_lane.sh --skip-tests" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to invoke service endpoint canonicalization contract lane coverage" >&2
  exit 1
fi

if ! grep -q "run_multikey_algorithm_policy_contract_lane.sh --skip-tests" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to invoke multikey algorithm policy contract lane coverage" >&2
  exit 1
fi

if ! grep -q "agent_interop_wave_docs" "$SHARED_CONTRACT"; then
  echo "expected did registry lane to include agent interop planning docs contract coverage" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected did registry contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected did registry contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected did registry wrapper to resolve did registry manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "did_registry_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected did registry manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "did registry contract lane script tests passed."
