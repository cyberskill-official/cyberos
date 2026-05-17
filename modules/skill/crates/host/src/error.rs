use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("skill not found: {0}")]
    NotFound(String),
    #[error("invalid manifest in {path}: {reason}")]
    InvalidManifest { path: String, reason: String },
    #[error("capability denied — skill `{skill}` requested `{cap}`")]
    CapabilityDenied { skill: String, cap: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
