#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SDK_LIB="$ROOT_DIR/crates/kamn-sdk/src/lib.rs"
SDK_TYPES="$ROOT_DIR/crates/kamn-sdk/src/types.rs"
SDK_AGENT="$ROOT_DIR/crates/kamn-sdk/src/agent.rs"

if ! grep -Fq "#![warn(missing_docs)]" "$SDK_LIB"; then
  echo "rustdoc policy contract failed: kamn-sdk is missing #![warn(missing_docs)]." >&2
  exit 1
fi

if ! grep -Fq 'must use the `kamn:did:agent:` prefix' "$SDK_TYPES"; then
  echo "rustdoc policy contract failed: AgentDid docs must mention the did prefix rule." >&2
  exit 1
fi

if ! grep -Fq "High-level KAMN agent workflow API." "$SDK_AGENT"; then
  echo "rustdoc policy contract failed: KamnAgent trait doc marker missing." >&2
  exit 1
fi

echo "rustdoc policy contract tests passed."
