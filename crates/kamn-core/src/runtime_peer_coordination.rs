use super::{
    DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureDecision,
    RuntimeBackpressureError, RuntimeBackpressureInput,
};
use crate::config::{NodeConfig, NodeRole};
use crate::signature_profile::{
    debug_fallback_signer_private_key_hex, service_auth_public_key_hex_from_private_key_hex,
    service_auth_sign_with_private_key_hex, service_auth_verify_with_public_key_hex,
};
use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};

const RUNTIME_PEER_FRAME_INVALID_SENDER_DID_REASON_CODE: &str =
    "runtime_peer_frame_invalid_sender_did";
const RUNTIME_PEER_FRAME_INVALID_RECIPIENT_DID_REASON_CODE: &str =
    "runtime_peer_frame_invalid_recipient_did";
const RUNTIME_PEER_FRAME_INVALID_LOCAL_PEER_DID_REASON_CODE: &str =
    "runtime_peer_frame_invalid_local_peer_did";
const RUNTIME_PEER_FRAME_SIGNING_PRIVATE_KEY_ENV: &str =
    "KAMN_RUNTIME_PEER_SIGNING_PRIVATE_KEY_HEX";
const RUNTIME_PEER_FRAME_SIGNING_PUBLIC_KEY_ENV: &str = "KAMN_RUNTIME_PEER_SIGNING_PUBLIC_KEY_HEX";

#[path = "runtime_peer_coordination/lifecycle_queue.rs"]
mod lifecycle_queue;
#[path = "runtime_peer_coordination/peer_frame.rs"]
mod peer_frame;
#[path = "runtime_peer_coordination/proposal_planning.rs"]
mod proposal_planning;
#[path = "runtime_peer_coordination/runtime_wiring.rs"]
mod runtime_wiring;

pub use lifecycle_queue::{
    BoundedRuntimeQueue, PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState,
    RuntimeLifecycleError, RuntimeQueueError,
};
pub use peer_frame::{AuthenticatedPeerFrame, AuthenticatedPeerFrameError, PeerFrameAuthenticator};
pub use proposal_planning::{
    DeterministicProposalPlanner, ProposalCandidate, ProposalPlan, ProposalPlannerError,
};
pub use runtime_wiring::{
    build_runtime_wiring, build_runtime_wiring_with_transport_profile, libp2p_feature_gate_name,
    resolve_libp2p_compile_mode, Libp2pCompileMode, RuntimeTransportProfile, RuntimeWiring,
};

#[cfg(test)]
#[path = "runtime_peer_coordination/tests.rs"]
mod tests;
