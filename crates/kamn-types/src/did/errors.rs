use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when parsing or validating a [`super::KamnDid`].
pub enum KamnDidError {
    /// DID input was empty after trimming.
    EmptyValue,
    /// DID did not start with required KAMN DID prefix.
    InvalidPrefix(String),
    /// DID segments were malformed.
    InvalidShape(String),
}

impl fmt::Display for KamnDidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue => write!(f, "kamn did must not be empty"),
            Self::InvalidPrefix(value) => write!(f, "invalid kamn did prefix: {value}"),
            Self::InvalidShape(value) => write!(f, "invalid kamn did shape: {value}"),
        }
    }
}

impl std::error::Error for KamnDidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when parsing or validating an [`super::AgentDid`].
pub enum AgentDidError {
    /// DID did not start with the required KAMN agent prefix.
    InvalidPrefix(String),
    /// DID prefix was present but method-specific id was missing.
    MissingMethodSpecificId,
    /// Method-specific id contained unsupported characters.
    InvalidCharacter(String),
}

impl fmt::Display for AgentDidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(value) => write!(f, "invalid agent did prefix: {value}"),
            Self::MissingMethodSpecificId => {
                write!(f, "agent did method-specific id must not be empty")
            }
            Self::InvalidCharacter(value) => {
                write!(f, "agent did has invalid characters: {value}")
            }
        }
    }
}

impl std::error::Error for AgentDidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when validating or generating DID/public-key bindings.
pub enum AgentDidKeyBindingError {
    /// DID does not include key-binding fingerprint suffix.
    MissingKeyBinding,
    /// DID method-specific-id input is invalid for binding generation.
    InvalidMethodSpecificId(String),
    /// Public key hex could not be decoded.
    InvalidPublicKeyHex,
    /// DID fingerprint does not match derived public-key fingerprint.
    KeyBindingMismatch {
        /// Fingerprint embedded in DID.
        expected: String,
        /// Fingerprint derived from provided public key.
        actual: String,
    },
}

impl fmt::Display for AgentDidKeyBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKeyBinding => write!(f, "agent did key binding is missing"),
            Self::InvalidMethodSpecificId(reason) => {
                write!(f, "invalid agent did method-specific id: {reason}")
            }
            Self::InvalidPublicKeyHex => write!(f, "invalid public key hex for did key binding"),
            Self::KeyBindingMismatch { expected, actual } => write!(
                f,
                "agent did key binding mismatch: expected={expected}, actual={actual}"
            ),
        }
    }
}

impl std::error::Error for AgentDidKeyBindingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wrapper error for canonical DID parse helpers.
pub enum SharedDidParseError {
    /// Input was empty after canonical trim.
    EmptyInput,
    /// Underlying agent DID parse failure.
    Agent(AgentDidError),
    /// Underlying generic KAMN DID parse failure.
    Kamn(KamnDidError),
}

impl fmt::Display for SharedDidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "did input must not be empty"),
            Self::Agent(error) => write!(f, "agent did parse failed: {error}"),
            Self::Kamn(error) => write!(f, "kamn did parse failed: {error}"),
        }
    }
}

impl std::error::Error for SharedDidParseError {}
