//! reqwest-backed Forwarder + backend base URLs (FR-OBS-002 production shell).

use crate::error::{Backend, ProxyError};
use crate::handler::Forwarder;

/// Per-backend base URLs, read from env at boot (with in-cluster defaults).
#[derive(Clone, Debug)]
pub struct BackendUrls {
    pub prometheus: String,
    pub loki: String,
    pub tempo: String,
}

impl BackendUrls {
    pub fn from_env() -> Self {
        Self {
            prometheus: env_or("OBS_PROMETHEUS_URL", "http://prometheus:9090"),
            loki: env_or("OBS_LOKI_URL", "http://loki:3100"),
            tempo: env_or("OBS_TEMPO_URL", "http://tempo:3200"),
        }
    }

    pub fn base(&self, backend: Backend) -> &str {
        match backend {
            Backend::Prometheus => &self.prometheus,
            Backend::Loki => &self.loki,
            Backend::Tempo => &self.tempo,
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Forwards a GET to `<base><path>?<params>` and returns the response body. A connect/transport error
/// becomes `BackendUnreachable` (-> 503), never a 500.
pub struct HttpForwarder {
    client: reqwest::Client,
    urls: BackendUrls,
}

impl HttpForwarder {
    pub fn new(urls: BackendUrls) -> Self {
        Self {
            client: reqwest::Client::new(),
            urls,
        }
    }
}

impl Forwarder for HttpForwarder {
    async fn forward(
        &self,
        backend: Backend,
        path: &str,
        params: &[(String, String)],
    ) -> Result<String, ProxyError> {
        let url = format!("{}{}", self.urls.base(backend), path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| ProxyError::BackendUnreachable(e.to_string()))?;
        resp.text()
            .await
            .map_err(|e| ProxyError::BackendUnreachable(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_maps_per_backend() {
        let u = BackendUrls {
            prometheus: "http://p".into(),
            loki: "http://l".into(),
            tempo: "http://t".into(),
        };
        assert_eq!(u.base(Backend::Prometheus), "http://p");
        assert_eq!(u.base(Backend::Loki), "http://l");
        assert_eq!(u.base(Backend::Tempo), "http://t");
    }
}
