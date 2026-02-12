use crate::{AgentDid, ChannelModelError, ChannelStore, ChannelType};
use std::collections::BTreeMap;
use std::fmt;

/// Marketplace listing metadata published by a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceListing {
    /// Stable listing identifier.
    pub listing_id: String,
    /// Provider DID that owns the listing.
    pub provider_did: String,
    /// Human-readable service name.
    pub service_name: String,
    /// Service category label.
    pub category: String,
    /// Search tags attached to the listing.
    pub tags: Vec<String>,
    /// Hourly rate in atomic units.
    pub hourly_rate: u128,
    /// Marketplace negotiation channel identifier.
    pub negotiation_channel_id: String,
}

/// Optional filters applied to listing searches.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarketplaceSearchFilter {
    /// Optional exact category filter.
    pub category: Option<String>,
    /// Optional tag filter.
    pub tag: Option<String>,
    /// Optional maximum hourly rate.
    pub max_hourly_rate: Option<u128>,
}

/// Negotiation metadata produced when opening a thread for a listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationThreadHook {
    /// Listing identifier used for negotiation.
    pub listing_id: String,
    /// Negotiation channel identifier bound to the listing.
    pub negotiation_channel_id: String,
    /// Provider DID for the listing.
    pub provider_did: String,
    /// Requester DID opening the negotiation.
    pub requester_did: String,
}

/// In-memory marketplace engine for listing registration and lookup.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceMarketplaceEngine {
    listings: BTreeMap<String, ServiceListing>,
}

impl ServiceMarketplaceEngine {
    /// Creates an empty marketplace engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new listing after validating shape, DID, and channel policy.
    pub fn register_listing(
        &mut self,
        listing: ServiceListing,
        channels: &ChannelStore,
    ) -> Result<(), ServiceMarketplaceError> {
        validate_listing_shape(&listing)?;
        validate_did(&listing.provider_did)?;

        if self.listings.contains_key(&listing.listing_id) {
            return Err(ServiceMarketplaceError::DuplicateListing(
                listing.listing_id.clone(),
            ));
        }

        let channel_type = channels
            .channel_type(&listing.negotiation_channel_id)
            .map_err(map_channel_error)?;
        if channel_type != ChannelType::Marketplace {
            return Err(ServiceMarketplaceError::NegotiationChannelType {
                channel_id: listing.negotiation_channel_id.clone(),
                found: channel_type,
            });
        }

        let provider_member = channels
            .is_member(&listing.negotiation_channel_id, &listing.provider_did)
            .map_err(map_channel_error)?;
        if !provider_member {
            return Err(ServiceMarketplaceError::ProviderNotChannelMember {
                provider_did: listing.provider_did.clone(),
                channel_id: listing.negotiation_channel_id.clone(),
            });
        }

        self.listings.insert(listing.listing_id.clone(), listing);
        Ok(())
    }

    /// Returns listings that match the provided search filter.
    pub fn search(&self, filter: &MarketplaceSearchFilter) -> Vec<ServiceListing> {
        self.listings
            .values()
            .filter(|listing| matches_filter(listing, filter))
            .cloned()
            .collect()
    }

    /// Opens a negotiation thread hook for a listing and requester DID.
    pub fn open_negotiation_thread(
        &self,
        listing_id: &str,
        requester_did: &str,
    ) -> Result<NegotiationThreadHook, ServiceMarketplaceError> {
        if listing_id.trim().is_empty() {
            return Err(ServiceMarketplaceError::EmptyField("listing_id"));
        }
        validate_did(requester_did)?;

        let listing = self
            .listings
            .get(listing_id)
            .ok_or_else(|| ServiceMarketplaceError::ListingNotFound(listing_id.to_owned()))?;
        Ok(NegotiationThreadHook {
            listing_id: listing.listing_id.clone(),
            negotiation_channel_id: listing.negotiation_channel_id.clone(),
            provider_did: listing.provider_did.clone(),
            requester_did: requester_did.to_owned(),
        })
    }
}

/// Errors produced by marketplace validation, registration, and lookup flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceMarketplaceError {
    /// Channel lookup failed.
    ChannelLookup(String),
    /// Listing id already exists.
    DuplicateListing(String),
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID validation failed.
    InvalidDid(String),
    /// Hourly rate was invalid.
    InvalidHourlyRate(u128),
    /// Listing was not found.
    ListingNotFound(String),
    /// Negotiation channel has the wrong channel type.
    NegotiationChannelType {
        /// Channel identifier.
        channel_id: String,
        /// Observed channel type.
        found: ChannelType,
    },
    /// Provider is not a member of the negotiation channel.
    ProviderNotChannelMember {
        /// Provider DID.
        provider_did: String,
        /// Channel identifier.
        channel_id: String,
    },
}

impl fmt::Display for ServiceMarketplaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChannelLookup(error) => write!(f, "channel lookup error: {error}"),
            Self::DuplicateListing(listing_id) => write!(f, "duplicate listing id: {listing_id}"),
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(error) => write!(f, "invalid did: {error}"),
            Self::InvalidHourlyRate(value) => {
                write!(f, "hourly rate must be greater than zero, found {value}")
            }
            Self::ListingNotFound(listing_id) => write!(f, "listing not found: {listing_id}"),
            Self::NegotiationChannelType { channel_id, found } => write!(
                f,
                "negotiation channel {channel_id} must be Marketplace, found {found:?}"
            ),
            Self::ProviderNotChannelMember {
                provider_did,
                channel_id,
            } => write!(
                f,
                "provider {provider_did} is not a member of channel {channel_id}"
            ),
        }
    }
}

impl std::error::Error for ServiceMarketplaceError {}

fn validate_listing_shape(listing: &ServiceListing) -> Result<(), ServiceMarketplaceError> {
    if listing.listing_id.trim().is_empty() {
        return Err(ServiceMarketplaceError::EmptyField("listing_id"));
    }
    if listing.service_name.trim().is_empty() {
        return Err(ServiceMarketplaceError::EmptyField("service_name"));
    }
    if listing.category.trim().is_empty() {
        return Err(ServiceMarketplaceError::EmptyField("category"));
    }
    if listing.tags.is_empty() {
        return Err(ServiceMarketplaceError::EmptyField("tags"));
    }
    if listing.tags.iter().any(|tag| tag.trim().is_empty()) {
        return Err(ServiceMarketplaceError::EmptyField("tag"));
    }
    if listing.hourly_rate == 0 {
        return Err(ServiceMarketplaceError::InvalidHourlyRate(
            listing.hourly_rate,
        ));
    }
    if listing.negotiation_channel_id.trim().is_empty() {
        return Err(ServiceMarketplaceError::EmptyField(
            "negotiation_channel_id",
        ));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), ServiceMarketplaceError> {
    AgentDid::parse(value)
        .map_err(|error| ServiceMarketplaceError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn matches_filter(listing: &ServiceListing, filter: &MarketplaceSearchFilter) -> bool {
    if let Some(expected_category) = filter.category.as_deref() {
        if listing.category != expected_category {
            return false;
        }
    }

    if let Some(expected_tag) = filter.tag.as_deref() {
        if !listing.tags.iter().any(|tag| tag == expected_tag) {
            return false;
        }
    }

    if let Some(max_rate) = filter.max_hourly_rate {
        if listing.hourly_rate > max_rate {
            return false;
        }
    }

    true
}

fn map_channel_error(error: ChannelModelError) -> ServiceMarketplaceError {
    ServiceMarketplaceError::ChannelLookup(error.to_string())
}
