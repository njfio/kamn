use crate::{AgentDid, ChannelModelError, ChannelStore, ChannelType};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceListing {
    pub listing_id: String,
    pub provider_did: String,
    pub service_name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub hourly_rate: u128,
    pub negotiation_channel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarketplaceSearchFilter {
    pub category: Option<String>,
    pub tag: Option<String>,
    pub max_hourly_rate: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationThreadHook {
    pub listing_id: String,
    pub negotiation_channel_id: String,
    pub provider_did: String,
    pub requester_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceMarketplaceEngine {
    listings: BTreeMap<String, ServiceListing>,
}

impl ServiceMarketplaceEngine {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn search(&self, filter: &MarketplaceSearchFilter) -> Vec<ServiceListing> {
        self.listings
            .values()
            .filter(|listing| matches_filter(listing, filter))
            .cloned()
            .collect()
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceMarketplaceError {
    ChannelLookup(String),
    DuplicateListing(String),
    EmptyField(&'static str),
    InvalidDid(String),
    InvalidHourlyRate(u128),
    ListingNotFound(String),
    NegotiationChannelType {
        channel_id: String,
        found: ChannelType,
    },
    ProviderNotChannelMember {
        provider_did: String,
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
