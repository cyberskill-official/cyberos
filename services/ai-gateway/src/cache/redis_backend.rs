//! FR-AI-017 §3 — Redis backend for per-tenant cache.

use std::sync::OnceLock;
use std::time::Duration;

use redis::AsyncCommands;
use tokio::time::timeout;

use super::{CacheError, CacheLookupOutcome, CacheInsertOutcome, CachedResponse, REDIS_TIMEOUT_MS};

static REDIS_URL: OnceLock<String> = OnceLock::new();

/// Initialize the Redis connection URL. Call once at startup.
pub fn init(url: &str) {
    REDIS_URL.set(url.to_string()).ok();
}

async fn get_conn() -> Result<redis::aio::ConnectionManager, redis::RedisError> {
    let url = REDIS_URL
        .get()
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::InvalidClientConfig, "redis URL not initialized")))?;
    let client = redis::Client::open(url.as_str())?;
    let conn = redis::aio::ConnectionManager::new(client).await?;
    Ok(conn)
}

/// §1 #1: lookup with timeout, schema-version check, and metrics.
pub async fn lookup(key: &super::CacheKey) -> CacheLookupOutcome {
    let t0 = std::time::Instant::now();
    let mut conn = match timeout(Duration::from_millis(REDIS_TIMEOUT_MS), get_conn()).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            super::error_metric("unreachable");
            return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
        Err(_) => {
            super::error_metric("timeout");
            return CacheLookupOutcome::Error(CacheError::Timeout);
        }
    };

    let raw: Result<Option<Vec<u8>>, _> = conn.get(key.redis_key()).await;
    let bytes = match raw {
        Ok(Some(b)) => b,
        Ok(None) => {
            super::lookup_metric(&key.tenant_id, "miss", t0.elapsed());
            return CacheLookupOutcome::Miss;
        }
        Err(e) => {
            super::error_metric("unreachable");
            return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
    };

    // §1 #13: schema-version check on load.
    match serde_json::from_slice::<CachedResponse>(&bytes) {
        Ok(cr) if cr.schema_version == super::CACHE_SCHEMA_VERSION => {
            super::lookup_metric(&key.tenant_id, "hit", t0.elapsed());
            CacheLookupOutcome::Hit(Box::new(cr), t0.elapsed())
        }
        Ok(_) => {
            super::lookup_metric(&key.tenant_id, "schema_mismatch", t0.elapsed());
            CacheLookupOutcome::SchemaMismatch
        }
        Err(e) => {
            super::error_metric("deserialisation");
            CacheLookupOutcome::Error(CacheError::Deserialisation(e.to_string()))
        }
    }
}

/// §1 #4 + §1 #5: insert with jittered TTL.
pub async fn insert(
    key: &super::CacheKey,
    bytes: &[u8],
    jittered_ttl: Duration,
) -> CacheInsertOutcome {
    let mut conn = match timeout(Duration::from_millis(REDIS_TIMEOUT_MS), get_conn()).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            super::error_metric("unreachable");
            return CacheInsertOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
        Err(_) => {
            super::error_metric("timeout");
            return CacheInsertOutcome::Error(CacheError::Timeout);
        }
    };

    let _: Result<(), _> = conn
        .set_ex(key.redis_key(), bytes, jittered_ttl.as_secs())
        .await;

    CacheInsertOutcome::Inserted {
        ttl: jittered_ttl,
        jittered_ttl,
    }
}
