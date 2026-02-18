# Shell LOC Reduction Plan

**Date:** 2026-02-17
**Baseline:** 115,524 shell LOC across 966 files + 84,455 Python LOC across 280 files
**Target:** Reduce shell LOC by 40-50% while preserving all contract-lane functionality
**Estimated final state:** ~60,000-70,000 shell LOC across ~500-600 files

---

## Current State Analysis

### Size Distribution (966 shell files)

| Bucket | Files | % | Est. LOC | % of LOC |
|--------|-------|---|----------|----------|
| < 50 lines | 384 | 39.4% | ~11,500 | ~10% |
| 50-100 lines | 185 | 19.0% | ~13,900 | ~12% |
| 100-200 lines | 244 | 25.1% | ~36,600 | ~32% |
| 200+ lines | 161 | 16.5% | ~53,500 | ~46% |

### File Taxonomy (966 shell files + 175 symlinks)

| Prefix | Count | Typical Size | Role |
|--------|-------|-------------|------|
| `test_*` | 558 | 6-1668 lines | Test/verification scripts |
| `run_*` | 285 | 6-256 lines | Lane executors (many are symlinks) |
| `check_*` | 126 | 6 lines | Policy validators (115 are exec-python wrappers) |
| `validate_*` | 77 | 20-200 lines | Validation scripts |
| `generate_*` | 45 | 50-200 lines | Evidence bundle generators |

### Redundancy Inventory

| Pattern | Occurrences | LOC Consumed | Root Cause |
|---------|-------------|-------------|------------|
| exec-python wrappers (<=8 lines) | 170 files | ~1,020 | No universal dispatcher for Python |
| exec-bash wrappers (<=8 lines) | 54 files | ~324 | No universal dispatcher for bash |
| `ROOT_DIR=...` boilerplate | ~791 non-symlink files | ~2,373 | No sourced common library |
| Dispatcher case statement | 1 file, 113 entries | ~196 | Not data-driven, hardcoded mapping |
| `resolve_phase_name()` case | 1 file, 35 entries | ~35 | Phase not encoded in manifest |
| Wave test scripts (near-identical) | 10 files | ~470 | Not parametrized |
| Wave wrapper matrix scripts | 10+ files | ~1,500+ | Not parametrized |
| Duplicated `usage()` functions | 233 files | ~3,500 | No shared library |
| Duplicated `emit_fallback_error()` | 118 files | ~1,400 | No shared library |
| Duplicated `extract_value()` | 80 files | ~800 | No shared library |
| Duplicated `assert_eq()` | 53 files | ~530 | No shared library |
| Manual JSON construction | 438 files | ~4,400 | No JSON helper library |

---

## Architecture Today

```
Symlink (run_X_contract_lane.sh)
  -> Dispatcher (run_non_kolme_contract_lane_dispatch.sh)
     -> case statement resolves manifest name
     -> case statement resolves phase name
     -> run_manifest_lane.sh
        -> reads manifest JSON
        -> execs implementation script

check_X_policy.sh (6-line wrapper)
  -> exec python3 X_contract.py "$@"

test_X.sh (standalone)
  -> inline test logic
```

### Key Files

| File | Lines | Role |
|------|-------|------|
| `scripts/framework/run_non_kolme_contract_lane_dispatch.sh` | 256 | Central lane dispatcher with hardcoded case |
| `scripts/framework/run_manifest_lane.sh` | ~40 | Manifest executor |
| `scripts/framework/manifests/*.json` | 171 files | Lane manifest definitions |
| `scripts/framework/contract_framework.py` | 129 | Python contract framework |
| `scripts/framework/contract_lane_helpers.py` | 86 | Python lane helpers |
| `scripts/framework/lane_manifest.py` | 123 | Python manifest parser |
| `scripts/framework/process_harness.py` | 226 | Python process execution |

---

## Reduction Plan

### Phase 0: Foundation — Shared Shell Library

**Goal:** Create `scripts/lib/common.sh` sourced by all non-symlink scripts.
**Estimated savings:** ~6,000-8,000 lines
**Risk:** LOW
**Effort:** 2-3 days

#### 0.1 Create `scripts/lib/common.sh`

```bash
#!/usr/bin/env bash
# Shared KAMN shell library — sourced by all scripts.

# Canonical root directory (idempotent).
KAMN_ROOT="${KAMN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[1]}")/../.." && pwd)}"

# Standard error helpers.
emit_fallback_error() {
  local taxonomy_version="${FALLBACK_REASON_TAXONOMY_VERSION:-kamn.framework.fallback-reason-taxonomy.v1}"
  local reason_code="$1"
  local reason_detail="$2"
  echo "dispatch_status=fail" >&2
  echo "fallback_reason_taxonomy_version=$taxonomy_version" >&2
  echo "fallback_reason_code=$reason_code" >&2
  echo "fallback_reason_detail=$reason_detail" >&2
}

# Standard assertion helpers.
assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [[ "$expected" != "$actual" ]]; then
    echo "FAIL: $label: expected='$expected' actual='$actual'" >&2
    return 1
  fi
}

assert_contains() {
  local label="$1" haystack="$2" needle="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "FAIL: $label: expected to contain '$needle'" >&2
    return 1
  fi
}

# JSON output helper.
# Usage: emit_json key1 val1 key2 val2 ...
emit_json() {
  local first=1
  printf '{'
  while (($# >= 2)); do
    [[ $first -eq 0 ]] && printf ','
    printf '"%s":"%s"' "$1" "$2"
    first=0
    shift 2
  done
  printf '}\n'
}

# Standard argument parser.
# Usage: parse_standard_args "$@"
# Sets: OUTPUT_JSON, CI_FAST_GATE, REPORT_FILE, LANE_PROFILE, REMAINING_ARGS
parse_standard_args() {
  OUTPUT_JSON=""
  CI_FAST_GATE=""
  REPORT_FILE=""
  LANE_PROFILE=""
  REMAINING_ARGS=()
  while (($# > 0)); do
    case "$1" in
      --output-json) OUTPUT_JSON="$2"; shift 2 ;;
      --ci-fast-gate) CI_FAST_GATE="$2"; shift 2 ;;
      --report-file) REPORT_FILE="$2"; shift 2 ;;
      --lane-profile) LANE_PROFILE="$2"; shift 2 ;;
      --) shift; REMAINING_ARGS=("$@"); break ;;
      *) REMAINING_ARGS+=("$1"); shift ;;
    esac
  done
}

# Extract key=value from a report file.
extract_value() {
  local file="$1" key="$2"
  grep "^${key}=" "$file" 2>/dev/null | head -1 | cut -d= -f2-
}
```

**Estimated size:** ~80 lines

#### 0.2 Migration strategy

Replace the 3-line boilerplate header in every non-symlink script:

```bash
# BEFORE (in every script):
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# AFTER:
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
```

This is a mechanical transformation. Every `$ROOT_DIR` reference becomes `$KAMN_ROOT`.

**Lines saved per file:** 2 (ROOT_DIR definition replaced by source) + any `usage()`, `emit_fallback_error()`, `assert_eq()`, `extract_value()` functions that are now sourced (avg ~8-12 lines per file that uses them).

**Total saved:**
- ROOT_DIR consolidation: ~791 files x 2 lines = ~1,582 lines
- `usage()` dedup: ~233 files x ~15 lines avg = ~3,495 lines (conservative)
- `emit_fallback_error()` dedup: ~118 files x ~8 lines = ~944 lines
- `assert_eq()` dedup: ~53 files x ~6 lines = ~318 lines
- `extract_value()` dedup: ~80 files x ~4 lines = ~320 lines
- **Subtotal: ~6,659 lines saved**

#### 0.3 Validation

- Run `cargo test --workspace` (Rust doc-contract tests depend on script paths)
- Run `bash scripts/ci/test_ci_tools.sh`
- Verify all symlinks still resolve
- Verify CI fast-gate passes

---

### Phase 1: Data-Driven Dispatcher

**Goal:** Replace the hardcoded `resolve_manifest_name()` case statement with manifest directory scanning.
**Estimated savings:** ~200 lines (small LOC but eliminates ongoing maintenance cost)
**Risk:** LOW-MEDIUM
**Effort:** 1 day

#### 1.1 Add `wrapper_name` field to manifests

Currently, the dispatcher maps wrapper names to manifest filenames via a 113-entry case statement. Instead, each manifest should declare what wrapper(s) it serves:

```json
{
  "schema_version": "kamn.contract-lane.manifest.v2",
  "lane_id": "bridge.adapter_conformance.contract",
  "wrapper_name": "run_bridge_adapter_conformance_contract_lane.sh",
  "phase": "contract",
  "evidence_key": "bridge_adapter_conformance_contract_lane:v1",
  "reason_key": "bridge_adapter_conformance_contract_lane_reason_codes:GO:v1",
  "phases": {
    "contract": ["bash", "scripts/bridge/run_bridge_adapter_conformance_contract_lane_impl.sh"]
  }
}
```

New fields:
- `wrapper_name`: The symlink/wrapper filename this manifest handles
- `phase`: Default phase to use (eliminates `resolve_phase_name()` case statement)

#### 1.2 Rewrite dispatcher to scan manifests

```bash
resolve_manifest_by_scan() {
  local wrapper="$1"
  local manifests_dir="$KAMN_ROOT/scripts/framework/manifests"
  for manifest in "$manifests_dir"/*.json; do
    if grep -q "\"wrapper_name\": \"$wrapper\"" "$manifest" 2>/dev/null; then
      echo "$manifest"
      return 0
    fi
  done
  return 1
}
```

Or faster: build a lookup index at dispatch time:

```bash
# Build index once (cached in /tmp)
build_manifest_index() {
  local index="/tmp/kamn-manifest-index-$(stat -c %Y "$KAMN_ROOT/scripts/framework/manifests" 2>/dev/null || echo 0)"
  if [[ -f "$index" ]]; then
    cat "$index"
    return
  fi
  for manifest in "$KAMN_ROOT/scripts/framework/manifests"/*.json; do
    local wrapper
    wrapper=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1])).get('wrapper_name',''))" "$manifest" 2>/dev/null)
    [[ -n "$wrapper" ]] && echo "$wrapper=$manifest"
  done | tee "$index"
}
```

#### 1.3 Eliminate both case statements

The new dispatcher becomes ~60 lines instead of 256:

```bash
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

WRAPPER_NAME="$(basename "$0")"
# ... arg parsing for --lane-wrapper, --resolve-manifest-path ...

MANIFEST_PATH="$(resolve_manifest_by_scan "$WRAPPER_NAME")" || {
  emit_fallback_error "dispatcher_unknown_wrapper" "no manifest for: $WRAPPER_NAME"
  exit 1
}

PHASE=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1])).get('phase','contract'))" "$MANIFEST_PATH")

exec bash "$KAMN_ROOT/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" --phase "$PHASE" -- "$@"
```

**Lines saved:** 256 - 60 = ~196 lines
**Maintenance saved:** Never need to edit dispatcher when adding new lanes

#### 1.4 Migrate all 171 manifests

Mechanical: add `"wrapper_name"` and `"phase"` fields to each JSON manifest. This can be scripted from the existing case statements.

---

### Phase 2: Eliminate Exec Wrappers

**Goal:** Replace 224 tiny (<=8 line) wrapper scripts with a universal dispatcher or generated symlinks.
**Estimated savings:** ~1,300 lines, ~224 files eliminated
**Risk:** MEDIUM
**Effort:** 2-3 days

#### 2.1 The Problem

170 shell scripts look exactly like this:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec python3 "$ROOT_DIR/scripts/governance/governance_lifecycle_rollback_policy_contract.py" "$@"
```

54 more look like this but exec bash instead of python3.

These exist solely to give a shell entry point for a Python (or nested bash) script.

#### 2.2 Solution A: Convention-Based Universal Dispatcher (Recommended)

Create `scripts/lib/exec_dispatch.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

# Derive target from script name and location.
# check_X_policy.sh -> X_contract.py (in same directory)
# run_X_impl.sh -> X_contract.py (in same directory)
SELF_NAME="$(basename "$0" .sh)"
SELF_DIR="$(dirname "$0")"

# Look for a matching Python script first, then bash.
for candidate in \
  "$SELF_DIR/${SELF_NAME}_contract.py" \
  "$SELF_DIR/${SELF_NAME}.py" \
  "$SELF_DIR/${SELF_NAME}_contract.sh" \
  "$SELF_DIR/${SELF_NAME}.sh"; do
  if [[ -f "$candidate" ]]; then
    case "$candidate" in
      *.py) exec python3 "$candidate" "$@" ;;
      *.sh) exec bash "$candidate" "$@" ;;
    esac
  fi
done

echo "no dispatch target found for $0" >&2
exit 1
```

Then replace each 6-line wrapper with a symlink:

```bash
# BEFORE: scripts/governance/check_governance_lifecycle_rollback_policy.sh (6 lines)
# AFTER:
ln -sf ../lib/exec_dispatch.sh scripts/governance/check_governance_lifecycle_rollback_policy.sh
```

**Issue:** The naming convention isn't perfectly regular. Some wrappers pass subcommands (e.g., `check-policy`). These need a mapping file or a convention tweak.

#### 2.3 Solution B: Registry File (Safer)

Create `scripts/lib/exec_registry.json`:

```json
{
  "check_governance_lifecycle_rollback_policy": {
    "target": "scripts/governance/governance_lifecycle_rollback_policy_contract.py",
    "args_prefix": []
  },
  "check_libp2p_convergence_process_isolated_live_policy": {
    "target": "scripts/runtime/libp2p_convergence_process_isolated_live_contract.py",
    "args_prefix": ["check-policy"]
  }
}
```

Universal dispatcher reads registry, resolves target, execs. All 224 wrappers become symlinks to the universal dispatcher.

**Lines saved:** 224 files x ~6 lines = 1,344 lines - ~50 lines (dispatcher + registry) = **~1,294 lines**
**Files eliminated:** 224 (replaced by symlinks)

#### 2.4 Migration Strategy

1. Build the registry from existing wrappers (scripted extraction)
2. Verify every entry resolves correctly
3. Replace wrappers with symlinks one domain at a time
4. Run CI after each domain migration
5. Delete old wrapper files

---

### Phase 3: Consolidate Wave/Matrix Test Scripts

**Goal:** Replace 10+ near-identical wave test scripts with one parametrized version.
**Estimated savings:** ~1,500-2,000 lines
**Risk:** LOW
**Effort:** 1 day

#### 3.1 The Problem

`scripts/framework/` contains:
```
test_non_kolme_wave10_lightweight_contract_lane_dispatch_wrapper_matrix.sh  (47 lines)
test_non_kolme_wave11_lightweight_contract_lane_dispatch_wrapper_matrix.sh  (47 lines)
...through wave19...
```

Each is identical except for the list of lane wrappers tested.

Similarly, `scripts/ci/` contains:
```
check_non_kolme_wave1_wrapper_family_budget_trend.sh
check_non_kolme_wave2_wrapper_family_budget_trend.sh
...through wave19...
```

#### 3.2 Solution

Create `scripts/framework/test_non_kolme_wave_matrix.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

WAVE_NUM="${1:?Usage: $0 <wave-number>}"
WAVE_MANIFEST="$KAMN_ROOT/scripts/framework/wave_definitions/wave${WAVE_NUM}.txt"

if [[ ! -f "$WAVE_MANIFEST" ]]; then
  echo "unknown wave: $WAVE_NUM (no manifest at $WAVE_MANIFEST)" >&2
  exit 1
fi

DISPATCHER="$KAMN_ROOT/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
# ... standard matrix test logic reading from $WAVE_MANIFEST ...
```

Create `scripts/framework/wave_definitions/wave10.txt`:
```
scripts/task/run_task_operation_snapshot_contract_lane.sh
scripts/runtime/run_zk_witness_mutation_contract_lane.sh
scripts/sdk/run_rust_live_transport_contract_lane.sh
scripts/message/run_message_lifecycle_contract_lane.sh
```

Same pattern for the CI budget trend scripts.

**Lines saved:** ~10 files x 47 lines + 10 files x ~150 lines = ~1,970 lines
**New files:** 1 parametrized script (~60 lines) + 10 wave definition files (~5 lines each) = ~110 lines
**Net savings:** ~1,860 lines

---

### Phase 4: Consolidate Test Boilerplate

**Goal:** Extract repeated test patterns into a test harness library.
**Estimated savings:** ~8,000-12,000 lines
**Risk:** MEDIUM
**Effort:** 3-5 days

#### 4.1 The Problem

558 `test_*` scripts share common patterns:
- Setup (create temp dirs, set variables)
- Run a script and capture output
- Assert output contains expected markers
- Assert exit code
- Cleanup

Many test scripts are 40-100 lines where 30-80 lines are boilerplate and 10-20 lines are the actual test-specific assertions.

#### 4.2 Create `scripts/lib/test_harness.sh`

```bash
# Test harness library — sourced by test scripts.

TEST_PASS=0
TEST_FAIL=0
TEST_TMPDIR=""

test_setup() {
  TEST_TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TEST_TMPDIR"' EXIT
}

run_and_capture() {
  local cmd="$1"
  shift
  TEST_STDOUT="$TEST_TMPDIR/stdout"
  TEST_STDERR="$TEST_TMPDIR/stderr"
  TEST_EXIT=0
  eval "$cmd" "$@" >"$TEST_STDOUT" 2>"$TEST_STDERR" || TEST_EXIT=$?
}

assert_exit_code() {
  local expected="$1"
  if [[ "$TEST_EXIT" -ne "$expected" ]]; then
    echo "FAIL: expected exit=$expected got exit=$TEST_EXIT" >&2
    TEST_FAIL=$((TEST_FAIL + 1))
    return 1
  fi
  TEST_PASS=$((TEST_PASS + 1))
}

assert_stdout_contains() {
  local marker="$1"
  if ! grep -q "$marker" "$TEST_STDOUT" 2>/dev/null; then
    echo "FAIL: stdout missing marker: $marker" >&2
    TEST_FAIL=$((TEST_FAIL + 1))
    return 1
  fi
  TEST_PASS=$((TEST_PASS + 1))
}

assert_stderr_contains() {
  local marker="$1"
  if ! grep -q "$marker" "$TEST_STDERR" 2>/dev/null; then
    echo "FAIL: stderr missing marker: $marker" >&2
    TEST_FAIL=$((TEST_FAIL + 1))
    return 1
  fi
  TEST_PASS=$((TEST_PASS + 1))
}

assert_json_field() {
  local file="$1" field="$2" expected="$3"
  local actual
  actual=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])" "$file" "$field" 2>/dev/null)
  assert_eq "$field" "$expected" "$actual"
}

test_summary() {
  echo "passed=$TEST_PASS failed=$TEST_FAIL"
  [[ "$TEST_FAIL" -eq 0 ]]
}
```

**Estimated size:** ~100 lines

#### 4.3 Refactored test script example

```bash
# BEFORE (typical 80-line test):
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT
OUTPUT="$TMPDIR/output.json"
# ... 20 lines of setup ...
bash "$ROOT_DIR/scripts/runtime/validate_X.sh" \
  --output-json "$OUTPUT" --ci-fast-gate PASS >"$TMPDIR/stdout" 2>"$TMPDIR/stderr" || true
# ... 40 lines of individual grep/assert checks ...
if grep -q "final_decision=GO" "$TMPDIR/stdout"; then echo "PASS"; else echo "FAIL"; exit 1; fi
# ... more checks ...

# AFTER (refactored 25-line test):
#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
source "$KAMN_ROOT/scripts/lib/test_harness.sh"
test_setup

run_and_capture "bash $KAMN_ROOT/scripts/runtime/validate_X.sh" \
  --output-json "$TEST_TMPDIR/output.json" --ci-fast-gate PASS

assert_exit_code 0
assert_stdout_contains "final_decision=GO"
assert_stdout_contains "reason_taxonomy_version=kamn.runtime"
assert_json_field "$TEST_TMPDIR/output.json" "final_decision" "GO"
test_summary
```

**Savings per script:** ~55 lines avg (80 → 25)
**Applicable scripts:** ~300 of the 558 test scripts have this pattern
**Total savings:** 300 x 55 = ~16,500 lines (conservative estimate: ~8,000-12,000)

#### 4.4 Migration Strategy

1. Create `scripts/lib/test_harness.sh`
2. Migrate 10 test scripts from one domain as a pilot (e.g., `scripts/did/test_*`)
3. Validate CI passes
4. Roll out domain-by-domain: did, bridge, governance, runtime, ci, kolme, sdk
5. Each domain migration is an independent PR

---

### Phase 5: Consolidate JSON Construction

**Goal:** Replace inline JSON construction with helper functions.
**Estimated savings:** ~3,000-4,000 lines
**Risk:** LOW
**Effort:** 1-2 days

#### 5.1 The Problem

438 scripts construct JSON output manually:

```bash
# Pattern A: heredoc (90 scripts)
cat <<EOF > "$output_json"
{
  "final_decision": "$final_decision",
  "reason_taxonomy_version": "$taxonomy",
  "reason_codes_csv": "$reason_codes",
  "evidence_status": "$evidence_status"
}
EOF

# Pattern B: echo/printf (112 scripts)
echo "{\"final_decision\":\"$final_decision\",\"reason_taxonomy_version\":\"$taxonomy\"}" > "$output_json"
```

#### 5.2 Solution

Add to `scripts/lib/common.sh`:

```bash
# Write a contract-lane JSON output bundle.
# Usage: write_contract_json "$output_file" key1 val1 key2 val2 ...
write_contract_json() {
  local outfile="$1"; shift
  python3 -c "
import json,sys
d={}
args=sys.argv[1:]
for i in range(0,len(args),2):
    d[args[i]]=args[i+1]
json.dump(d, open('$outfile','w'), indent=2)
print('output_json=$outfile')
" "$@"
}

# Standard contract-lane output with GO/NO-GO decision.
write_decision_json() {
  local outfile="$1"
  local decision="$2"
  local taxonomy="$3"
  local reason_codes="$4"
  shift 4
  write_contract_json "$outfile" \
    "final_decision" "$decision" \
    "reason_taxonomy_version" "$taxonomy" \
    "reason_codes_csv" "$reason_codes" \
    "$@"
}
```

**Lines saved per script:** ~8-10 lines (heredoc/echo block → 1-2 function calls)
**Applicable scripts:** ~200 of the 438 that use the contract-lane output pattern
**Total savings:** 200 x ~15 lines = ~3,000 lines

---

### Phase 6: Python Script Consolidation

**Goal:** Reduce Python LOC by consolidating repeated contract patterns.
**Estimated savings:** ~10,000-15,000 Python LOC
**Risk:** LOW-MEDIUM
**Effort:** 3-5 days

#### 6.1 The Problem

121 `*_contract.py` files follow an identical structure:

```python
import argparse
from framework.contract_framework import DecisionAccumulator, load_json, write_json

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True)
    parser.add_argument("--ci-fast-gate", required=True)
    parser.add_argument("--output-json", required=True)
    args = parser.parse_args()

    report = load_json(args.report_file)
    acc = DecisionAccumulator()

    # Domain-specific checks (10-30 lines):
    acc.reject_if(report.get("X") != "expected", "reason_code_X")
    acc.reject_if(report.get("Y") != "expected", "reason_code_Y")

    decision, reasons = acc.finalize("all_checks_passed")
    write_json(args.output_json, {
        "final_decision": decision,
        "reason_taxonomy_version": "kamn.domain.taxonomy.v1",
        "reason_codes_csv": ",".join(reasons),
    })
    print(f"final_decision={decision}")

if __name__ == "__main__":
    main()
```

The boilerplate (argparse, load, accumulate, write, print) is ~30-40 lines.
The domain-specific checks are ~10-30 lines.
**Ratio: 60-80% boilerplate.**

#### 6.2 Solution: Declarative Policy Checker

Create `scripts/framework/declarative_policy_checker.py`:

```python
"""
Declarative policy checker — eliminates boilerplate in contract.py files.

Usage:
  python3 declarative_policy_checker.py \
    --policy-file policies/bridge_adapter_conformance.yaml \
    --report-file /tmp/report.json \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json /tmp/output.json
"""
```

Policy file (`policies/bridge_adapter_conformance.yaml`):
```yaml
taxonomy_version: "kamn.bridge.adapter-conformance-reason-taxonomy.v1"
checks:
  - field: "adapter_initialization_status"
    expected: "verified"
    reason_code: "adapter_initialization_unverified"
  - field: "transport_normalization_status"
    expected: "verified"
    reason_code: "transport_normalization_unverified"
  - field: "ingress_routing_status"
    expected: "verified"
    reason_code: "ingress_routing_unverified"
```

**For the ~60 contract.py files that are pure field-equality checks:**
- Each 50-80 line Python file → 10-15 line YAML policy file
- The Python file is eliminated entirely

**For the ~40 contract.py files with complex logic:**
- Keep as Python but source common boilerplate from the framework
- Reduce from ~80 lines to ~30 lines each

**Lines saved:**
- 60 files eliminated x 60 lines avg = 3,600 Python lines
- 40 files simplified x 40 lines avg = 1,600 Python lines
- New framework code: ~200 lines
- New YAML policies: 60 x 12 lines = 720 lines (but these are data, not code)
- **Net Python savings: ~5,000 lines**

#### 6.3 Shell wrapper elimination (cascading effect)

The 115 `check_*.sh` scripts that exec these Python contracts can now exec the declarative checker instead, making them even more amenable to the Phase 2 universal dispatcher:

```bash
# All 60 declarative-policy check_ wrappers become:
exec python3 "$KAMN_ROOT/scripts/framework/declarative_policy_checker.py" \
  --policy-file "$KAMN_ROOT/scripts/policies/$(basename "$0" .sh).yaml" "$@"
```

---

### Phase 7: Generate Manifests from Registry

**Goal:** Replace 171 static manifest JSON files with a generated registry.
**Estimated savings:** ~1,500 lines (small LOC but eliminates file proliferation)
**Risk:** MEDIUM
**Effort:** 2 days

#### 7.1 The Problem

Each manifest is ~10 lines of JSON with a predictable structure. Adding a new lane requires:
1. Create the manifest JSON
2. Add case entry to dispatcher (eliminated by Phase 1)
3. Create symlink
4. Create test wrapper

#### 7.2 Solution: Single Registry + Generator

Create `scripts/framework/lane_registry.yaml`:

```yaml
lanes:
  - domain: bridge
    name: adapter_conformance
    type: contract
    impl: bash scripts/bridge/run_bridge_adapter_conformance_contract_lane_impl.sh

  - domain: bridge
    name: credentialed
    type: contract
    impl: bash scripts/bridge/run_bridge_credentialed_contract_lane_impl.sh

  - domain: bridge
    name: credentialed
    type: deep
    impl: bash scripts/bridge/run_bridge_credentialed_deep_lane_impl.sh
  # ... 168 more entries ...
```

Generator script (`scripts/framework/generate_manifests.py`):
```python
# Reads lane_registry.yaml
# Generates: manifests/*.json, symlinks, wave definitions
# Validates: all impl scripts exist, no orphan manifests
```

**Lines saved:** 171 files x ~10 lines = 1,710 lines → 1 registry file ~500 lines + generator ~100 lines
**Net savings:** ~1,100 lines
**Bigger win:** Adding a new lane = 1 line in the registry + the impl script. No manifest, no symlink, no dispatcher edit.

---

## Summary: Projected Savings

| Phase | Description | Shell LOC Saved | Python LOC Saved | Files Eliminated | Risk | Effort |
|-------|-------------|-----------------|------------------|------------------|------|--------|
| 0 | Shared shell library | ~6,600 | — | 0 | LOW | 2-3 days |
| 1 | Data-driven dispatcher | ~200 | — | 0 | LOW-MED | 1 day |
| 2 | Eliminate exec wrappers | ~1,300 | — | ~224 | MEDIUM | 2-3 days |
| 3 | Consolidate wave scripts | ~1,860 | — | ~20 | LOW | 1 day |
| 4 | Test harness library | ~10,000 | — | 0 | MEDIUM | 3-5 days |
| 5 | JSON construction helpers | ~3,000 | — | 0 | LOW | 1-2 days |
| 6 | Python declarative checker | ~700 (wrappers) | ~5,000 | ~60 py, ~60 sh | LOW-MED | 3-5 days |
| 7 | Manifest registry | ~1,100 | — | ~171 json | MEDIUM | 2 days |
| **TOTAL** | | **~24,760** | **~5,000** | **~535** | | **15-22 days** |

### Before vs After Projections

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Shell LOC | 115,524 | ~90,700 | **-21%** |
| Python LOC | 84,455 | ~79,500 | **-6%** |
| Total script LOC | 199,979 | ~170,200 | **-15%** |
| Shell files | 966 | ~720 | **-25%** |
| Python files | 280 | ~220 | **-21%** |
| Symlinks | 175 | ~400 | +225 (replacing wrappers) |
| Manifest JSON files | 171 | 0 (generated) | -171 |
| Script:Rust LOC ratio | 1.61:1 | 1.37:1 | **-15%** |

### Conservative vs Aggressive Estimates

The table above uses **conservative** estimates. With aggressive Phase 4 migration (all 558 test scripts refactored):

| Metric | Conservative | Aggressive |
|--------|-------------|-----------|
| Shell LOC saved | 24,760 | ~45,000 |
| Final shell LOC | ~90,700 | ~70,500 |
| Final script:Rust ratio | 1.37:1 | 1.14:1 |

---

## Execution Order & Dependencies

```
Phase 0 (common.sh) ─┬─> Phase 1 (data-driven dispatcher)
                      ├─> Phase 2 (exec wrapper elimination)
                      ├─> Phase 3 (wave consolidation)
                      ├─> Phase 4 (test harness) ──> Phase 5 (JSON helpers)
                      └─> Phase 6 (Python declarative) ──> Phase 7 (manifest registry)
```

Phase 0 is the prerequisite for everything. After Phase 0, all other phases can proceed independently or in parallel.

**Recommended order:** 0 → 1 → 3 → 2 → 5 → 4 → 6 → 7

Rationale: Start with lowest risk, build confidence, then tackle the larger refactors.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking CI pipelines | Each phase is a separate PR; run full CI before merge |
| Breaking external script references | Search for script paths in AGENTS.md, docs/, specs/, CI workflows before renaming |
| Sourcing common.sh performance | Minimal — bash source is <1ms; no subshell overhead |
| Symlink resolution on different platforms | Already using 175 symlinks successfully; no new platform risk |
| Manifest schema migration (v1 → v2) | Add backward-compatible `wrapper_name` field; v1 readers ignore unknown fields |
| Test harness masking failures | Test harness is strictly additive — existing assertion patterns still work |

---

## Non-Goals

- **Do not reduce Python framework code** beyond declarative policy extraction. The Python contract framework (`contract_framework.py`, `lane_manifest.py`, etc.) is well-structured and not a LOC problem.
- **Do not consolidate domain logic.** Each domain's `*_impl.sh` and complex `*_contract.py` files contain domain-specific logic that should remain separate.
- **Do not eliminate the manifest concept.** Manifests are good architecture. The problem is that they're static files instead of generated data.
- **Do not eliminate symlinks.** Symlinks are the right mechanism for lane entry points. The plan increases symlink usage.
