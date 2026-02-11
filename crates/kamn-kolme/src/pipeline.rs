//! Runtime-commit pipeline scaffold for Kolme extraction.

use std::error::Error;
use std::fmt;

use crate::codec::{KolmeCodecError, KolmeWireCodec};
use crate::finality::{resolve_finality, FinalityState};
use crate::transport::{KolmeTransport, TransportError, TransportRequest};

/// Pipeline-level error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    /// Codec conversion failed.
    Codec(KolmeCodecError),
    /// Transport submission failed.
    Transport(TransportError),
    /// Finality threshold not yet satisfied.
    FinalityPending {
        /// Observed confirmations.
        confirmations: u64,
        /// Required confirmations.
        threshold: u64,
    },
    /// Finality was explicitly rejected.
    FinalityRejected,
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codec(err) => write!(f, "codec error: {err}"),
            Self::Transport(err) => write!(f, "transport error: {err}"),
            Self::FinalityPending {
                confirmations,
                threshold,
            } => write!(
                f,
                "finality pending: confirmations={confirmations}, threshold={threshold}"
            ),
            Self::FinalityRejected => f.write_str("finality rejected"),
        }
    }
}

impl Error for PipelineError {}

impl From<KolmeCodecError> for PipelineError {
    fn from(value: KolmeCodecError) -> Self {
        Self::Codec(value)
    }
}

impl From<TransportError> for PipelineError {
    fn from(value: TransportError) -> Self {
        Self::Transport(value)
    }
}

/// Deterministic runtime-commit pipeline scaffold.
pub struct RuntimeCommitPipeline<C, T> {
    codec: C,
    transport: T,
    finality_threshold: u64,
}

impl<C, T> RuntimeCommitPipeline<C, T>
where
    C: KolmeWireCodec,
    T: KolmeTransport,
{
    /// Creates a pipeline with default single-confirmation finality.
    pub fn new(codec: C, transport: T) -> Self {
        Self {
            codec,
            transport,
            finality_threshold: 1,
        }
    }

    /// Configures finality threshold used by submit.
    pub fn with_finality_threshold(mut self, threshold: u64) -> Self {
        self.finality_threshold = threshold.max(1);
        self
    }

    /// Encodes payload, submits transport request, and validates finality.
    pub fn submit(
        &self,
        endpoint: &str,
        payload: &[u8],
        confirmations: u64,
        rejected: bool,
    ) -> Result<Vec<u8>, PipelineError> {
        let encoded = self.codec.encode(payload)?;
        let response = self
            .transport
            .submit(TransportRequest::new(endpoint, encoded))?;

        if response.status >= 400 {
            return Err(PipelineError::Transport(TransportError::RejectedStatus(
                response.status,
            )));
        }

        let finality = resolve_finality(confirmations, self.finality_threshold, rejected);
        match finality.state() {
            FinalityState::Confirmed => self.codec.decode(&response.body).map_err(Into::into),
            FinalityState::Pending => Err(PipelineError::FinalityPending {
                confirmations: finality.confirmations(),
                threshold: finality.threshold(),
            }),
            FinalityState::Rejected => Err(PipelineError::FinalityRejected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PipelineError, RuntimeCommitPipeline};
    use crate::codec::PassthroughCodec;
    use crate::transport::EchoTransport;

    #[test]
    fn functional_pipeline_submit_roundtrips_payload_when_confirmed() {
        let pipeline = RuntimeCommitPipeline::new(PassthroughCodec, EchoTransport);
        let payload = b"signed-runtime-commit";
        let output = pipeline
            .submit("http://kolme.local/tx", payload, 1, false)
            .expect("submit should succeed");
        assert_eq!(output, payload);
    }

    #[test]
    fn unit_pipeline_submit_fails_when_finality_pending() {
        let pipeline =
            RuntimeCommitPipeline::new(PassthroughCodec, EchoTransport).with_finality_threshold(2);

        let error = pipeline
            .submit("http://kolme.local/tx", b"payload", 1, false)
            .expect_err("pending finality should fail");

        assert_eq!(
            error,
            PipelineError::FinalityPending {
                confirmations: 1,
                threshold: 2
            }
        );
    }
}
