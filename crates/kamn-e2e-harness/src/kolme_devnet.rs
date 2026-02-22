/// Kolme devnet settings scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeDevnetConfig {
    /// API endpoint.
    pub api_url: String,
    /// Notifications websocket URL.
    pub ws_url: String,
}

impl Default for KolmeDevnetConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_owned(),
            ws_url: "ws://localhost:3000/notifications".to_owned(),
        }
    }
}
