/// Supported external bridge platforms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BridgePlatform {
    /// Telegram bridge surface.
    Telegram,
    /// Discord bridge surface.
    Discord,
    /// Slack bridge surface.
    Slack,
    /// Signal bridge surface.
    Signal,
    /// X/Twitter bridge surface.
    Twitter,
    /// Custom bridge surface identified by label.
    Custom(String),
}

impl BridgePlatform {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Telegram => "telegram".to_owned(),
            Self::Discord => "discord".to_owned(),
            Self::Slack => "slack".to_owned(),
            Self::Signal => "signal".to_owned(),
            Self::Twitter => "twitter".to_owned(),
            Self::Custom(name) => name.trim().to_lowercase(),
        }
    }
}

/// Direction of bridge traffic used by policy and audit surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BridgeDirection {
    /// External platform to KAMN ingress.
    Inbound,
    /// KAMN to external platform egress.
    Outbound,
}
