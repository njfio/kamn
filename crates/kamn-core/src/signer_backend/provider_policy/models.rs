/// Backend-tagged signature payload returned by routing/signing APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSignature {
    /// Backend identifier that produced this signature.
    pub backend: String,
    /// Signature material returned by the backend.
    pub signature: String,
}
