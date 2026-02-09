# Reputation Signal Routing (Issue #210 / #211)

This slice wires endorsement, dispute, and capability-verification signals into deterministic reputation query/routing outputs.

## Signal Integration Model
- Input state source: `ReputationStore` (`AgentReputation` records).
- Signal contributors:
  - endorsements
  - disputes
  - verified capabilities
- APIs:
  - `rank_agents_for_routing(...)`
  - `rank_listings_by_reputation(...)`

Routing adjustment model:
- positive: endorsements and verified-capability volume
- negative: dispute count
- capability gate:
  - all required capabilities matched => capability bonus
  - missing any required capability => capability penalty

## Query and Routing Outputs
- Agent ranking returns:
  - base trust score
  - signal adjustment
  - final routing score
  - summary counts + matched capabilities
- Listing ranking returns:
  - listing/provider IDs
  - base trust score
  - signal adjustment
  - final routing score
  - matched capabilities

Tie scores are resolved by DID lexical order.

## Validation and Error Handling
- Invalid candidate DID values are rejected before ranking.
- Missing reputation records are rejected explicitly.
- Duplicate agent candidates in a single ranking request are rejected.
- Empty required-capability entries are rejected.
- Invalid (negative) signal weights are rejected.

deterministic tie-break uses agent DID lexical order.

## Dispute Evidence Contract (Issue #737 / #738)
Routing-affecting dispute outcomes are published through deterministic evidence bundles before score corrections are accepted.

- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh --output-file /tmp/reputation-dispute.json --dispute-id dispute-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --dispute-reason-code QUALITY --evidence-uri s3://kamn-audit/reputation/dispute-001.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --evidence-hash-verified PASS --original-trust-score 640 --proposed-trust-score 560 --max-adjustment-points 120 --policy-window-open true --approval-recorded true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_dispute_policy.sh --bundle-file /tmp/reputation-dispute.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_dispute_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json`
- Replay matrix runner:
  - `python3 scripts/reputation/run_reputation_dispute_matrix.py --fixture fixtures/reputation_dispute/replay_cases.json --output-json reputation-dispute-report.json`
- Regression policy:
  - tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).

## Fast and Cost-Effective Validation
Run the targeted lane first:

```bash
cargo test -p kamn-core --test reputation_signal_routing --test reputation_signal_routing_docs
bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh
bash scripts/reputation/test_run_reputation_dispute_matrix.sh
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
bash scripts/reputation/test_run_reputation_dispute_deep_lane.sh
cargo test -p kamn-core
```
