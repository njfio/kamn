#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_critical_path_coverage_gate.sh [options]

Runs a bounded critical-path coverage probe for core+node modules and enforces
coverage thresholds using scripts/ci/check_critical_path_coverage.py.

Options:
  --threshold-file <path>    Coverage threshold file.
  --core-json <path>         Output coverage JSON for kamn-core probe.
  --node-json <path>         Output coverage JSON for kamn-node probe.
  --output-json <path>       Output coverage policy report JSON.
USAGE
}

threshold_file=".ci/critical-path-coverage-thresholds.json"
core_json="ci-critical-path-core-coverage.json"
node_json="ci-critical-path-node-coverage.json"
output_json="ci-critical-path-coverage-policy.json"

while (($# > 0)); do
  case "$1" in
    --threshold-file)
      threshold_file="${2:-}"
      shift 2
      ;;
    --core-json)
      core_json="${2:-}"
      shift 2
      ;;
    --node-json)
      node_json="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required" >&2
  exit 2
fi
if ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "cargo llvm-cov is required; install via cargo install cargo-llvm-cov --locked" >&2
  exit 2
fi

cargo llvm-cov clean --workspace

cargo llvm-cov -p kamn-core --lib --json --output-path "$core_json" -- \
  direct_message_crypto::tests::decrypt_rejects_algorithm_mismatch --exact
cargo llvm-cov -p kamn-core --lib --no-clean --json --output-path "$core_json" -- \
  direct_message_crypto::tests::encrypt_decrypt_roundtrip_succeeds_for_valid_payload --exact
cargo llvm-cov -p kamn-core --lib --no-clean --json --output-path "$core_json" -- \
  direct_message_crypto::tests::decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility --exact
cargo llvm-cov -p kamn-core --lib --no-clean --json --output-path "$core_json" -- \
  group_channel_crypto::tests::encrypt_requires_key_agreement_seed --exact
cargo llvm-cov -p kamn-core --lib --no-clean --json --output-path "$core_json" -- \
  group_channel_crypto::tests::encrypt_decrypt_roundtrip_requires_authorized_recipient --exact
cargo llvm-cov -p kamn-core --test kolme_runtime_commit_http_transport --no-clean --json --output-path "$core_json" -- \
  regression_http_transport_maps_401_to_authorization_unavailable_error --exact
cargo llvm-cov -p kamn-core --test kolme_runtime_commit_http_transport --no-clean --json --output-path "$core_json" -- \
  functional_http_transport_includes_authorization_header_when_configured --exact
cargo llvm-cov -p kamn-core --test kolme_runtime_commit_http_transport --no-clean --json --output-path "$core_json" -- \
  regression_http_transport_timeout_maps_to_provider_timeout --exact
cargo llvm-cov -p kamn-core --test kolme_runtime_commit_http_transport --no-clean --json --output-path "$core_json" -- \
  regression_https_transport_maps_certificate_errors_to_unavailable --exact

cargo llvm-cov clean --workspace

cargo llvm-cov -p kamn-node --bin kamn-node --json --output-path "$node_json" -- \
  main_tests::runtime_tests::unit_full_supervisor_stop_contract_classifier_rejects_status_mismatch --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  runtime_orchestration::tests::unit_full_supervisor_http_probe_accepts_success_status --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  runtime_orchestration::tests::unit_full_supervisor_inter_tick_probes_execute_once_per_lane --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  main_tests::service_api_endpoint_tests::unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::unit_nonce_retry_classifier_marks_transient_provider_errors --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::unit_nonce_retry_backoff_policy_is_deterministic_and_bounded --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::unit_signer_preflight_defaults_to_single_signer_quorum_ready --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::regression_signer_preflight_rejects_stale_failover_rotation_epoch --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::regression_signer_preflight_rejects_non_failover_rotation_epoch_regression --exact
cargo llvm-cov -p kamn-node --bin kamn-node --no-clean --json --output-path "$node_json" -- \
  signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer --exact

python3 scripts/ci/check_critical_path_coverage.py \
  --core-coverage-json "$core_json" \
  --node-coverage-json "$node_json" \
  --threshold-file "$threshold_file" \
  --output-json "$output_json"

echo "critical_path_coverage_core_report=$core_json"
echo "critical_path_coverage_node_report=$node_json"
echo "critical_path_coverage_policy_report=$output_json"
