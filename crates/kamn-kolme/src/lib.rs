//! `kamn-kolme` hosts the extracted Kolme runtime-commit boundary.
//!
//! This initial scaffold keeps the API surface intentionally small while
//! extraction from `kamn-core` is in flight.
#![warn(missing_docs)]

pub mod codec;
pub mod finality;
pub mod pipeline;
pub mod transport;

pub use codec::{KolmeCodecError, KolmeWireCodec, PassthroughCodec};
pub use finality::{resolve_finality, FinalityResolution, FinalityState};
pub use pipeline::{PipelineError, RuntimeCommitPipeline};
pub use transport::{
    EchoTransport, KolmeTransport, TransportError, TransportRequest, TransportResponse,
};

#[cfg(test)]
mod tests {
    use super::{resolve_finality, FinalityState};

    #[test]
    fn unit_scaffold_exports_finality_resolution() {
        let resolution = resolve_finality(1, 1, false);
        assert_eq!(resolution.state(), FinalityState::Confirmed);
        assert!(resolution.is_final());
    }
}
