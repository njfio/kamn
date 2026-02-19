use crate::{AgentDid, ReputationStore, ServiceListing};
use std::collections::BTreeSet;
use std::fmt;

const REPUTATION_SIGNAL_INVALID_CANDIDATE_DID_REASON_CODE: &str =
    "reputation_signal_invalid_candidate_did";
const REPUTATION_SIGNAL_INVALID_PROVIDER_DID_REASON_CODE: &str =
    "reputation_signal_invalid_provider_did";

/// Weight configuration used to convert reputation evidence into routing adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutingSignalWeights {
    /// Bonus per endorsement signal.
    pub endorsement_weight: i32,
    /// Penalty per dispute signal.
    pub dispute_penalty_weight: i32,
    /// Bonus applied when required capabilities are fully matched.
    pub capability_match_bonus: i32,
    /// Penalty applied when required capabilities are not fully matched.
    pub capability_miss_penalty: i32,
    /// Bonus per verified capability.
    pub verification_weight: i32,
}

impl Default for RoutingSignalWeights {
    fn default() -> Self {
        Self {
            endorsement_weight: 6,
            dispute_penalty_weight: 18,
            capability_match_bonus: 40,
            capability_miss_penalty: 30,
            verification_weight: 2,
        }
    }
}

/// Aggregated reputation signal counts used in candidate ranking explanations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReputationSignalSummary {
    /// Number of endorsements on record.
    pub endorsements: usize,
    /// Number of disputes on record.
    pub disputes: usize,
    /// Number of verified capabilities on record.
    pub verified_capabilities: usize,
    /// Required capabilities matched by the candidate.
    pub matched_capabilities: Vec<String>,
}

/// Ranked agent candidate for routing decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedAgentCandidate {
    /// Candidate agent DID.
    pub agent_did: String,
    /// Baseline trust score.
    pub trust_score: u32,
    /// Reputation signal adjustment applied to the baseline score.
    pub signal_adjustment: i32,
    /// Final routing score after adjustment and clamping.
    pub routing_score: i32,
    /// Signal breakdown used to compute adjustment.
    pub summary: ReputationSignalSummary,
}

/// Ranked service-listing candidate for marketplace discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedListingCandidate {
    /// Marketplace listing identifier.
    pub listing_id: String,
    /// Provider DID for the listing.
    pub provider_did: String,
    /// Baseline trust score.
    pub trust_score: u32,
    /// Reputation signal adjustment applied to the baseline score.
    pub signal_adjustment: i32,
    /// Final routing score after adjustment and clamping.
    pub routing_score: i32,
    /// Required capabilities matched by the listing provider.
    pub matched_capabilities: Vec<String>,
}

/// Errors emitted while ranking candidates by reputation signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationSignalError {
    /// No candidates were supplied.
    EmptyCandidates,
    /// Candidate DID failed validation.
    InvalidCandidateDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Candidate DID appeared more than once.
    DuplicateCandidateDid(String),
    /// Candidate has no reputation record.
    MissingReputation(String),
    /// Required capabilities input contains invalid entries.
    InvalidRequiredCapability,
    /// A weight value is negative.
    InvalidWeight(&'static str),
}

impl fmt::Display for ReputationSignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCandidates => write!(f, "at least one candidate is required"),
            Self::InvalidCandidateDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::DuplicateCandidateDid(agent_did) => {
                write!(f, "duplicate candidate did: {agent_did}")
            }
            Self::MissingReputation(agent_did) => {
                write!(f, "missing reputation record for {agent_did}")
            }
            Self::InvalidRequiredCapability => {
                write!(f, "required capabilities must not contain empty entries")
            }
            Self::InvalidWeight(field) => {
                write!(f, "{field} must be greater than or equal to zero")
            }
        }
    }
}

impl std::error::Error for ReputationSignalError {}

/// Ranks agent candidates using trust score plus weighted reputation signals.
pub fn rank_agents_for_routing(
    store: &ReputationStore,
    candidate_dids: &[&str],
    required_capabilities: &[&str],
    weights: RoutingSignalWeights,
) -> Result<Vec<RankedAgentCandidate>, ReputationSignalError> {
    validate_weights(weights)?;
    let required = normalize_required_capabilities(required_capabilities)?;

    if candidate_dids.is_empty() {
        return Err(ReputationSignalError::EmptyCandidates);
    }

    let mut seen = BTreeSet::new();
    let mut ranked = Vec::with_capacity(candidate_dids.len());
    for candidate_did in candidate_dids {
        let parsed = AgentDid::parse(candidate_did).map_err(|error| {
            ReputationSignalError::InvalidCandidateDid {
                field: "candidate_did",
                reason_code: REPUTATION_SIGNAL_INVALID_CANDIDATE_DID_REASON_CODE,
                detail: error.to_string(),
            }
        })?;
        if !seen.insert(parsed.as_str().to_owned()) {
            return Err(ReputationSignalError::DuplicateCandidateDid(
                parsed.as_str().to_owned(),
            ));
        }

        let reputation = store
            .get_agent(parsed.as_str())
            .ok_or_else(|| ReputationSignalError::MissingReputation(parsed.as_str().to_owned()))?;

        let (signal_adjustment, summary) =
            signal_adjustment_for_reputation(reputation, &required, weights);
        let routing_score = clamp_score(reputation.trust_score as i32 + signal_adjustment);

        ranked.push(RankedAgentCandidate {
            agent_did: parsed.as_str().to_owned(),
            trust_score: reputation.trust_score,
            signal_adjustment,
            routing_score,
            summary,
        });
    }

    ranked.sort_by(|left, right| {
        right
            .routing_score
            .cmp(&left.routing_score)
            .then_with(|| right.trust_score.cmp(&left.trust_score))
            .then_with(|| left.agent_did.cmp(&right.agent_did))
    });

    Ok(ranked)
}

/// Ranks marketplace listings by provider reputation and capability fit.
pub fn rank_listings_by_reputation(
    listings: &[ServiceListing],
    store: &ReputationStore,
    required_capabilities: &[&str],
    weights: RoutingSignalWeights,
) -> Result<Vec<RankedListingCandidate>, ReputationSignalError> {
    validate_weights(weights)?;
    let required = normalize_required_capabilities(required_capabilities)?;

    if listings.is_empty() {
        return Err(ReputationSignalError::EmptyCandidates);
    }

    let mut ranked = Vec::with_capacity(listings.len());
    for listing in listings {
        let parsed = AgentDid::parse(&listing.provider_did).map_err(|error| {
            ReputationSignalError::InvalidCandidateDid {
                field: "listing.provider_did",
                reason_code: REPUTATION_SIGNAL_INVALID_PROVIDER_DID_REASON_CODE,
                detail: error.to_string(),
            }
        })?;
        let reputation = store
            .get_agent(parsed.as_str())
            .ok_or_else(|| ReputationSignalError::MissingReputation(parsed.as_str().to_owned()))?;

        let (signal_adjustment, summary) =
            signal_adjustment_for_reputation(reputation, &required, weights);
        let routing_score = clamp_score(reputation.trust_score as i32 + signal_adjustment);

        ranked.push(RankedListingCandidate {
            listing_id: listing.listing_id.clone(),
            provider_did: listing.provider_did.clone(),
            trust_score: reputation.trust_score,
            signal_adjustment,
            routing_score,
            matched_capabilities: summary.matched_capabilities,
        });
    }

    ranked.sort_by(|left, right| {
        right
            .routing_score
            .cmp(&left.routing_score)
            .then_with(|| right.trust_score.cmp(&left.trust_score))
            .then_with(|| left.provider_did.cmp(&right.provider_did))
            .then_with(|| left.listing_id.cmp(&right.listing_id))
    });

    Ok(ranked)
}

fn signal_adjustment_for_reputation(
    reputation: &crate::AgentReputation,
    required_capabilities: &[String],
    weights: RoutingSignalWeights,
) -> (i32, ReputationSignalSummary) {
    let endorsement_bonus =
        reputation.endorsements.len().min(20) as i32 * weights.endorsement_weight;
    let dispute_penalty = reputation.disputes.len().min(20) as i32 * weights.dispute_penalty_weight;
    let verification_bonus =
        reputation.verified_capabilities.len().min(20) as i32 * weights.verification_weight;

    let verified_names = reputation
        .verified_capabilities
        .iter()
        .map(|entry| entry.capability.as_str())
        .collect::<BTreeSet<_>>();
    let matched_capabilities = required_capabilities
        .iter()
        .filter(|capability| verified_names.contains(capability.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let capability_component = if required_capabilities.is_empty() {
        0
    } else if matched_capabilities.len() == required_capabilities.len() {
        weights.capability_match_bonus
    } else {
        -weights.capability_miss_penalty
    };

    let signal_adjustment =
        endorsement_bonus + verification_bonus + capability_component - dispute_penalty;

    (
        signal_adjustment,
        ReputationSignalSummary {
            endorsements: reputation.endorsements.len(),
            disputes: reputation.disputes.len(),
            verified_capabilities: reputation.verified_capabilities.len(),
            matched_capabilities,
        },
    )
}

fn normalize_required_capabilities(
    required_capabilities: &[&str],
) -> Result<Vec<String>, ReputationSignalError> {
    let mut capabilities = Vec::with_capacity(required_capabilities.len());
    for value in required_capabilities {
        if value.trim().is_empty() {
            return Err(ReputationSignalError::InvalidRequiredCapability);
        }
        capabilities.push((*value).to_owned());
    }
    capabilities.sort();
    capabilities.dedup();
    Ok(capabilities)
}

fn clamp_score(score: i32) -> i32 {
    score.clamp(0, 1_000)
}

fn validate_weights(weights: RoutingSignalWeights) -> Result<(), ReputationSignalError> {
    if weights.endorsement_weight < 0 {
        return Err(ReputationSignalError::InvalidWeight("endorsement_weight"));
    }
    if weights.dispute_penalty_weight < 0 {
        return Err(ReputationSignalError::InvalidWeight(
            "dispute_penalty_weight",
        ));
    }
    if weights.capability_match_bonus < 0 {
        return Err(ReputationSignalError::InvalidWeight(
            "capability_match_bonus",
        ));
    }
    if weights.capability_miss_penalty < 0 {
        return Err(ReputationSignalError::InvalidWeight(
            "capability_miss_penalty",
        ));
    }
    if weights.verification_weight < 0 {
        return Err(ReputationSignalError::InvalidWeight("verification_weight"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_required_capabilities, ReputationSignalError};

    #[test]
    fn normalize_capability_set_is_sorted_and_deduplicated() {
        let normalized = normalize_required_capabilities(&["python", "market-analysis", "python"])
            .expect("normalization should succeed");
        assert_eq!(normalized, vec!["market-analysis", "python"]);
    }

    #[test]
    fn normalize_rejects_empty_capability() {
        assert_eq!(
            normalize_required_capabilities(&["market-analysis", ""]),
            Err(ReputationSignalError::InvalidRequiredCapability)
        );
    }
}
