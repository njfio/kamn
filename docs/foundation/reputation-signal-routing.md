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

## Fast and Cost-Effective Validation
Run the targeted lane first:

```bash
cargo test -p kamn-core --test reputation_signal_routing --test reputation_signal_routing_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
