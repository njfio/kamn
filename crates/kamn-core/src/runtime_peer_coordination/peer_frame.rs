#[path = "peer_frame/authenticator.rs"]
mod authenticator;
#[path = "peer_frame/errors.rs"]
mod errors;
#[path = "peer_frame/frame.rs"]
mod frame;
#[path = "peer_frame/signing.rs"]
mod signing;

pub use authenticator::PeerFrameAuthenticator;
pub use errors::AuthenticatedPeerFrameError;
pub use frame::AuthenticatedPeerFrame;
