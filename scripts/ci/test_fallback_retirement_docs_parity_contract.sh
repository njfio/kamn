#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

DOC_FILES=(
  "$ROOT_DIR/README.md"
  "$ROOT_DIR/docs/ci/strategy.md"
  "$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
)

required_markers=(
  "fallback_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
  "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
  "contracts.fallback_signer_secret_rejected_profile_class=production"
  "contracts.fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
  "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract"
  "fallback_signer_secret_present_violation"
  "fallback_signer_secret_checkpoint_reason_mismatch"
  "Regression: #2337"
)

for doc_file in "${DOC_FILES[@]}"; do
  if [ ! -f "$doc_file" ]; then
    echo "fallback retirement docs parity contract failed: missing doc file $doc_file" >&2
    exit 1
  fi
done

for marker in "${required_markers[@]}"; do
  missing_docs=()
  for doc_file in "${DOC_FILES[@]}"; do
    if ! grep -Fq -- "$marker" "$doc_file"; then
      missing_docs+=("$doc_file")
    fi
  done
  if [ "${#missing_docs[@]}" -ne 0 ]; then
    printf 'fallback retirement docs parity contract failed: marker %q missing in %s\n' \
      "$marker" \
      "$(IFS=,; echo "${missing_docs[*]}")" >&2
    exit 1
  fi
done

echo "fallback retirement docs parity contract tests passed."
