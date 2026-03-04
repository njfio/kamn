use super::{ConfigError, NodeRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OutputMode {
    pub(crate) kind: OutputModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputModeKind {
    Text,
    Json,
}

impl OutputMode {
    pub(crate) fn text() -> Self {
        Self {
            kind: OutputModeKind::Text,
        }
    }

    pub(crate) fn json() -> Self {
        Self {
            kind: OutputModeKind::Json,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "text" => Ok(Self::text()),
            "json" => Ok(Self::json()),
            other => Err(ConfigError::InvalidOutputMode(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeMode {
    pub(crate) kind: RuntimeModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeModeKind {
    Bootstrap,
    Planning,
    RecoveryCheck,
    Daemon,
    Api,
    Full,
    KolmeLive,
}

impl RuntimeMode {
    pub(crate) fn bootstrap() -> Self {
        Self {
            kind: RuntimeModeKind::Bootstrap,
        }
    }

    pub(crate) fn planning() -> Self {
        Self {
            kind: RuntimeModeKind::Planning,
        }
    }

    pub(crate) fn recovery_check() -> Self {
        Self {
            kind: RuntimeModeKind::RecoveryCheck,
        }
    }

    pub(crate) fn daemon() -> Self {
        Self {
            kind: RuntimeModeKind::Daemon,
        }
    }

    pub(crate) fn api() -> Self {
        Self {
            kind: RuntimeModeKind::Api,
        }
    }

    pub(crate) fn full() -> Self {
        Self {
            kind: RuntimeModeKind::Full,
        }
    }

    pub(crate) fn kolme_live() -> Self {
        Self {
            kind: RuntimeModeKind::KolmeLive,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "bootstrap" => Ok(Self::bootstrap()),
            "planning" => Ok(Self::planning()),
            "recovery-check" => Ok(Self::recovery_check()),
            "daemon" => Ok(Self::daemon()),
            "api" => Ok(Self::api()),
            "full" => Ok(Self::full()),
            "kolme-live" => Ok(Self::kolme_live()),
            other => Err(ConfigError::InvalidRuntimeMode(other.to_owned())),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self.kind {
            RuntimeModeKind::Bootstrap => "bootstrap",
            RuntimeModeKind::Planning => "planning",
            RuntimeModeKind::RecoveryCheck => "recovery-check",
            RuntimeModeKind::Daemon => "daemon",
            RuntimeModeKind::Api => "api",
            RuntimeModeKind::Full => "full",
            RuntimeModeKind::KolmeLive => "kolme-live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticsMode {
    Basic,
    Snapshot,
}

impl DiagnosticsMode {
    pub(crate) fn basic() -> Self {
        Self::Basic
    }

    pub(crate) fn snapshot() -> Self {
        Self::Snapshot
    }

    pub(crate) fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "basic" => Ok(Self::basic()),
            "snapshot" => Ok(Self::snapshot()),
            other => Err(ConfigError::InvalidDiagnosticsMode(other.to_owned())),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalProfile {
    Processor,
    Listener,
    Approver,
}

impl LocalProfile {
    fn metadata(self) -> (&'static str, NodeRole, &'static str) {
        match self {
            Self::Processor => ("local-processor", NodeRole::Processor, "./data/processor"),
            Self::Listener => ("local-listener", NodeRole::Listener, "./data/listener"),
            Self::Approver => ("local-approver", NodeRole::Approver, "./data/approver"),
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "local-processor" => Ok(Self::Processor),
            "local-listener" => Ok(Self::Listener),
            "local-approver" => Ok(Self::Approver),
            other => Err(ConfigError::InvalidNodeProfile(other.to_owned())),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        self.metadata().0
    }

    pub(crate) fn default_role(self) -> NodeRole {
        self.metadata().1
    }

    pub(crate) fn default_storage_dir(self) -> &'static str {
        self.metadata().2
    }
}
