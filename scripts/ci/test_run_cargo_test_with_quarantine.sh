#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/run_cargo_test_with_quarantine.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat > "$TMP_DIR/flaky-tests.txt" <<'REGISTRY'
# owner|test-id|issue|expiry|notes
qa|crate::tests::flaky_a|#180|2099-12-31|tracked quarantine entry
qa|crate::tests::flaky_b|#181|2099-12-31|another tracked entry
REGISTRY

dry_output="$(bash "$SCRIPT" --registry "$TMP_DIR/flaky-tests.txt" --dry-run -- cargo test -p kamn-core --test invariant_harness)"

if ! printf '%s\n' "$dry_output" | grep -q -- '--skip crate::tests::flaky_a'; then
  echo "expected dry-run output to include first quarantined test skip flag" >&2
  exit 1
fi

if ! printf '%s\n' "$dry_output" | grep -q -- '--skip crate::tests::flaky_b'; then
  echo "expected dry-run output to include second quarantined test skip flag" >&2
  exit 1
fi

cat > "$TMP_DIR/empty-flaky-tests.txt" <<'REGISTRY'
# owner|test-id|issue|expiry|notes
REGISTRY

dry_output_empty="$(bash "$SCRIPT" --registry "$TMP_DIR/empty-flaky-tests.txt" --dry-run -- cargo test -p kamn-core --test invariant_harness)"
if printf '%s\n' "$dry_output_empty" | grep -q -- '--skip '; then
  echo "did not expect skip flags when registry is empty" >&2
  exit 1
fi

set +e
bash "$SCRIPT" --registry "$TMP_DIR/flaky-tests.txt" --dry-run -- cargo clippy >/dev/null 2>&1
invalid_cmd_status=$?
set -e
if [ "$invalid_cmd_status" -eq 0 ]; then
  echo "expected non-cargo-test command to fail" >&2
  exit 1
fi

echo "run_cargo_test_with_quarantine tests passed."
