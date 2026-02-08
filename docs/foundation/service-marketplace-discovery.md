# Service Marketplace Listing and Discovery (Issue #188)

This document captures the first implementation slice for service marketplace listing registration, deterministic discovery filters, and negotiation thread hooks.

## Scope Delivered
- Added `crates/kamn-core/src/service_marketplace.rs` with:
  - `ServiceListing` model for provider service offers.
  - `MarketplaceSearchFilter` for deterministic listing selection.
  - `ServiceMarketplaceEngine` for registration, search, and negotiation hook creation.
  - `NegotiationThreadHook` for routing requester/provider negotiation to a marketplace channel.
  - `ServiceMarketplaceError` typed validation and lookup failures.
- Added integration tests in `crates/kamn-core/tests/service_marketplace.rs`.

## Listing Validation Rules
- Listing registration requires:
  - non-empty listing id, service name, category, and negotiation channel id.
  - at least one non-empty tag.
  - positive hourly rate.
  - valid provider DID.
  - negotiation channel type of `Marketplace`.
  - provider DID membership in the negotiation channel.

## Discovery and Negotiation Rules
- Search filter supports:
  - exact category match.
  - exact tag membership.
  - max hourly rate guard.
- Negotiation hooks:
  - require an existing listing id.
  - require valid requester DID.
  - deterministically return listing id, negotiation channel id, provider DID, and requester DID.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test service_marketplace
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
